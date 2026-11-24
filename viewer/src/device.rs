//! Background connection to the keyboard over Rynk (USB).
//!
//! A dedicated thread runs a single-threaded tokio runtime (needed by
//! `rynk-usb`/`nusb`). It finds the keyboard, reads layout + keymap once,
//! then follows the firmware's pushed topics (layer changes, Caps Lock,
//! battery). When the link drops it goes back to searching, so plugging the
//! dongle in and out just works.
//!
//! The UI reads [`KeyboardState`] and sends [`Command`]s; an editor can add
//! write commands (e.g. `SetKey`) here later.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use embassy_futures::select::{Either, select, select3};
use rynk::rmk_types::action::KeyAction;
use rynk::rmk_types::battery::BatteryStatus;
use rynk::rmk_types::led_indicator::LedIndicator;
use rynk::rmk_types::protocol::rynk::DeviceCapabilities;
use rynk::rmk_types::protocol::rynk::PeripheralStatus;
use rynk::{Client, RynkDevice, RynkHostError, TopicEvent};
use rynk_usb::UsbDevice;
use tokio::sync::mpsc;

/// How long to wait between USB scans while no keyboard is found.
const SCAN_INTERVAL: Duration = Duration::from_millis(1000);
/// Bound on the open + handshake step (the library has no timeout of its own).
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
/// Topics are best-effort, so state that has no topic (split halves) or may
/// have been missed is re-read this often.
const POLL_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Default, PartialEq)]
pub enum LinkStatus {
    #[default]
    Searching,
    Connecting(String),
    Connected(String),
    /// Last session ended with this error; we keep scanning.
    Lost(String),
}

/// One key's physical placement, in key units (center + size + rotation).
#[derive(Clone, Debug)]
pub struct KeyPlacement {
    pub row: u8,
    pub col: u8,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub rotation_deg: f32,
}

#[derive(Clone, Debug, Default)]
pub struct KeyboardState {
    pub link: LinkStatus,
    pub device_name: String,
    pub caps: Option<DeviceCapabilities>,
    pub layout: Vec<KeyPlacement>,
    /// Flat keymap: index = (layer * rows + row) * cols + col.
    pub keymap: Vec<KeyAction>,
    pub current_layer: u8,
    pub default_layer: u8,
    pub leds: Option<LedIndicator>,
    pub peripherals: Vec<PeripheralStatus>,
    /// True once layout + keymap have been read in this session.
    pub loaded: bool,
}

impl KeyboardState {
    pub fn action(&self, layer: u8, row: u8, col: u8) -> Option<&KeyAction> {
        let caps = self.caps.as_ref()?;
        let (rows, cols) = (caps.num_rows as usize, caps.num_cols as usize);
        self.keymap
            .get((layer as usize * rows + row as usize) * cols + col as usize)
    }
}

/// Requests from the UI to the connection thread.
#[derive(Debug)]
pub enum Command {
    /// Re-read layout and keymap (e.g. after reflashing).
    Reload,
}

pub type Shared = Arc<Mutex<KeyboardState>>;

/// Start the connection thread. `on_change` is called whenever the state
/// changes (the UI uses it to request a repaint).
pub fn spawn(on_change: impl Fn() + Send + 'static) -> (Shared, mpsc::UnboundedSender<Command>) {
    let shared: Shared = Arc::default();
    let (tx, rx) = mpsc::unbounded_channel();
    let thread_shared = shared.clone();
    std::thread::Builder::new()
        .name("rynk".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(connection_loop(thread_shared, rx, on_change));
        })
        .expect("spawn rynk thread");
    (shared, tx)
}

async fn connection_loop(shared: Shared, mut rx: mpsc::UnboundedReceiver<Command>, on_change: impl Fn()) {
    let update = |f: &dyn Fn(&mut KeyboardState)| {
        f(&mut shared.lock().unwrap());
        on_change();
    };

    loop {
        let device = match UsbDevice::discover().await {
            Ok(list) => list.into_iter().next(),
            Err(e) => {
                update(&|s| s.link = LinkStatus::Lost(format!("USB scan failed: {e}")));
                None
            }
        };
        let Some(device) = device else {
            update(&|s| {
                if !matches!(s.link, LinkStatus::Lost(_)) {
                    s.link = LinkStatus::Searching;
                }
            });
            tokio::time::sleep(SCAN_INTERVAL).await;
            continue;
        };

        let label = device.label();
        update(&|s| {
            s.link = LinkStatus::Connecting(label.clone());
            s.device_name = label.clone();
        });

        let err = match tokio::time::timeout(CONNECT_TIMEOUT, device.connect()).await {
            Err(_) => "handshake timed out".to_string(),
            Ok(Err(e)) => connect_error_hint(&e),
            Ok(Ok((client, mut driver))) => {
                update(&|s| s.link = LinkStatus::Connected(label.clone()));
                let session = run_session(&client, &shared, &mut rx, &on_change);
                match select(driver.run(&client), session).await {
                    Either::First(e) => e.to_string(),
                    Either::Second(Err(e)) => e.to_string(),
                    Either::Second(Ok(never)) => match never {},
                }
            }
        };
        update(&|s| {
            s.link = LinkStatus::Lost(err.clone());
            s.loaded = false;
        });
        tokio::time::sleep(SCAN_INTERVAL).await;
    }
}

/// Add a platform hint to the most common failure: no permission to open
/// the device.
fn connect_error_hint(e: &RynkHostError) -> String {
    let msg = e.to_string();
    if cfg!(target_os = "linux") && (msg.contains("ermission") || msg.contains("Access")) {
        format!("{msg} — install the udev rule from viewer/README.md")
    } else {
        msg
    }
}

enum Never {}

async fn run_session(
    client: &Client,
    shared: &Shared,
    rx: &mut mpsc::UnboundedReceiver<Command>,
    on_change: &impl Fn(),
) -> Result<Never, RynkHostError> {
    load_everything(client, shared).await?;
    on_change();

    let topics = async {
        loop {
            let event = client.next_topic().await;
            {
                let mut s = shared.lock().unwrap();
                match event {
                    TopicEvent::LayerChange(layer) => s.current_layer = layer,
                    TopicEvent::LedIndicatorChange(leds) => s.leds = Some(leds),
                    _ => continue,
                }
            }
            on_change();
        }
    };

    let poll = async {
        loop {
            tokio::time::sleep(POLL_INTERVAL).await;
            poll_status(client, shared).await?;
            on_change();
        }
        #[allow(unreachable_code)]
        Err::<Never, RynkHostError>(RynkHostError::Disconnected)
    };

    let commands = async {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Reload => {
                    shared.lock().unwrap().loaded = false;
                    on_change();
                    load_everything(client, shared).await?;
                    on_change();
                }
            }
        }
        // The UI dropped its sender: the app is closing. Park forever; the
        // process exits around us.
        std::future::pending::<Result<Never, RynkHostError>>().await
    };

    match select3(topics, poll, commands).await {
        embassy_futures::select::Either3::First(never) => match never {},
        embassy_futures::select::Either3::Second(r) | embassy_futures::select::Either3::Third(r) => r,
    }
}

/// Read everything the view needs: capabilities, layout, keymap, layer, LEDs.
async fn load_everything(client: &Client, shared: &Shared) -> Result<(), RynkHostError> {
    let caps = client.get_capabilities().await?;
    let layout = client.get_layout().await?;
    let keymap = if caps.bulk_transfer_supported {
        client.read_all_keymap().await?
    } else {
        let mut v = Vec::new();
        for layer in 0..caps.num_layers {
            for row in 0..caps.num_rows {
                for col in 0..caps.num_cols {
                    v.push(client.get_key(layer, row, col).await?);
                }
            }
        }
        v
    };
    let default_layer = client.get_default_layer().await?;
    let current_layer = client.get_current_layer().await?;
    let leds = client.get_led_indicator().await.ok();

    let placements = match layout.variants.get(layout.default_variant as usize).or(layout.variants.first()) {
        Some(variant) => variant
            .keys
            .iter()
            .map(|k| KeyPlacement {
                row: k.row,
                col: k.col,
                x: k.rect.x,
                y: k.rect.y,
                w: k.rect.w,
                h: k.rect.h,
                rotation_deg: k.r,
            })
            .collect(),
        // Firmware without a `[layout].map`: plain matrix grid.
        None => (0..caps.num_rows)
            .flat_map(|row| {
                (0..caps.num_cols).map(move |col| KeyPlacement {
                    row,
                    col,
                    x: col as f32 + 0.5,
                    y: row as f32 + 0.5,
                    w: 1.0,
                    h: 1.0,
                    rotation_deg: 0.0,
                })
            })
            .collect(),
    };

    {
        let mut s = shared.lock().unwrap();
        s.caps = Some(caps);
        s.layout = placements;
        s.keymap = keymap;
        s.default_layer = default_layer;
        s.current_layer = current_layer;
        s.leds = leds;
        s.loaded = true;
    }
    poll_status(client, shared).await
}

/// State without a reliable topic: split halves, and a layer resync.
async fn poll_status(client: &Client, shared: &Shared) -> Result<(), RynkHostError> {
    let caps = client.get_capabilities().await?;
    let mut peripherals = Vec::new();
    if caps.is_split {
        for slot in 0..caps.num_split_peripherals {
            peripherals.push(client.get_peripheral_status(slot).await?);
        }
    }
    let layer = client.get_current_layer().await?;
    let mut s = shared.lock().unwrap();
    s.peripherals = peripherals;
    s.current_layer = layer;
    Ok(())
}

/// `"87%"` for a battery that reports a level, `None` if there's no battery.
pub fn battery_text(b: &BatteryStatus) -> Option<String> {
    match b {
        BatteryStatus::Available { level: Some(l), .. } => Some(format!("{l}%")),
        BatteryStatus::Available { level: None, .. } => Some("?%".into()),
        BatteryStatus::Unavailable => None,
    }
}
