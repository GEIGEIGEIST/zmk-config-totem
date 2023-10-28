

// autopair macros
ZMK_BEHAVIOR(mbkt, macro,
    bindings = <&macro_tap &kp LBKT &kp RBKT &kp LEFT>;
)
ZMK_BEHAVIOR(mbrc, macro,
    bindings = <&macro_tap &kp LBRC &kp RBRC &kp LEFT>;
)
ZMK_BEHAVIOR(mpar, macro,
    bindings = <&macro_tap &kp LPAR &kp RPAR &kp LEFT>;
)
ZMK_BEHAVIOR(msqt, macro,
    bindings = <&macro_tap &kp SQT &kp SQT &kp LEFT>;
)
ZMK_BEHAVIOR(mdqt, macro,
    bindings = <&macro_tap &kp DQT &kp DQT &kp LEFT>;
)
// tap: '' | shift + tap: ""
ZMK_BEHAVIOR(msqt_morph, mod_morph,
    bindings = <&msqt>, <&mdqt>;
    mods = <(MOD_LSFT|MOD_RSFT)>;
)
// tab navigation macros
ZMK_BEHAVIOR(mtabnext, macro,
    bindings = <&macro_tap &kp G &kp T>;
)
ZMK_BEHAVIOR(mtabprev, macro,
    bindings =
        <&macro_tap &kp G &kp LS(T)>;
)

// tap: sticky-shift | shift + tap/ double-tap: caps-word | hold: shift
//ZMK_BEHAVIOR(smart_shft, mod_morph,
//    bindings = <&sk LSHFT>, <&caps_word>;
//    mods = <(MOD_LSFT)>;
//)


// Command+Tab swapper, requires PR #1366
ZMK_BEHAVIOR(swapper, tri_state,
    bindings = <&kt LGUI>, <&kp TAB>, <&kt LGUI>;
    ignored-key-positions = <LB2>, <20>; // ignored key should be position of SWAP_PREV
)

// tap: comma | shift + tap: semicolon | ctrl + shift + tap: <
ZMK_BEHAVIOR(comma_morph, mod_morph,
    mods = <(MOD_LSFT|MOD_RSFT)>;
    bindings = <&kp COMMA>, <&comma_inner_morph>;
)
ZMK_BEHAVIOR(comma_inner_morph, mod_morph,
    bindings = <&kp SEMICOLON>, <&kp LESS_THAN>;
    mods = <(MOD_LCTL|MOD_RCTL)>;
)

// tap: dot | shift + tap: colon | ctrl + shift + tap: >
ZMK_BEHAVIOR(dot_morph, mod_morph,
    bindings = <&kp DOT>, <&dot_inner_morph>;
    mods = <(MOD_LSFT|MOD_RSFT)>;
)
ZMK_BEHAVIOR(dot_inner_morph, mod_morph,
    bindings = <&kp COLON>, <&kp GREATER_THAN>;
    mods = <(MOD_LCTL|MOD_RCTL)>;
)

// tap: qmark | shift + tap: excl
ZMK_BEHAVIOR(qexcl, mod_morph,
    bindings = <&kp QMARK>, <&kp EXCL>;
    mods = <(MOD_LSFT|MOD_RSFT)>;
)


// tap: space | shift + tap: underscore | hold: activate layer
ZMK_BEHAVIOR(lt_spc, hold_tap,
    flavor = "balanced";
    tapping-term-ms = <200>;
    quick-tap-ms = <QUICK_TAP_MS>;
    bindings = <&mo>, <&spund>;
)
ZMK_BEHAVIOR(spund, mod_morph,
  bindings = <&kp SPACE>, <&kp UNDERSCORE>;
  mods = <(MOD_LSFT|MOD_RSFT)>;
)

// tap: backspace | lshft + tap: delete | rshft + tap: shift-delete | hold: activate layer
ZMK_BEHAVIOR(lt_del, hold_tap,
    flavor = "balanced";
    tapping-term-ms = <200>;
    quick-tap-ms = <QUICK_TAP_MS>;
    bindings = <&mo>, <&bs_del>;
)
ZMK_BEHAVIOR(bs_del, mod_morph,
    bindings = <&kp BSPC>, <&kp DEL>;
    mods = <(MOD_LSFT|MOD_RSFT)>;
    keep-mods = <MOD_RSFT>;
)

// tap: sticky shift | double-tap: sticky num-layer | hold: num-layer
ZMK_BEHAVIOR(shift_num, hold_tap,
    flavor = "balanced";
    tapping-term-ms = <200>;
    quick-tap-ms = <QUICK_TAP_MS>;
    bindings = <&mo>, <&sticky_num_dance>;
)
ZMK_BEHAVIOR(sticky_num_dance, tap_dance,
    tapping-term-ms = <300>;
    bindings = <&sk LSHFT>, <&sl NUM>;  // reverse this for sticky-num on single tap
)

// tap: caps word | double-tap: caps lock
ZMK_BEHAVIOR(caps_dance, tap_dance,
    tapping-term-ms = <200>;
    bindings = <&caps_word>, <&kp CAPS>;
)
ZMK_BEHAVIOR(comma_space, macro,
    bindings = <&macro_tap &kp COMMA &kp SPACE>;
)
// tap: space | double-tap: comma + space | shift + tap: underscore
ZMK_BEHAVIOR(comma_dance, tap_dance,
    tapping-term-ms = <200>;
    bindings = <&spund>, <&comma_space>;
)



// Shortcuts and aliases


/* Apple Globe key */
#define C_KEYBOARD_LAYOUT_SELECT (ZMK_HID_USAGE(HID_USAGE_CONSUMER, 0x029D))
#define GLOBE C_KEYBOARD_LAYOUT_SELECT
// As per PR 1938

// Spectacle controls
#define SPC_UP    &kp LA(LG(UP))         // Cycle through upper positions
#define SPC_DOWN  &kp LA(LG(DOWN))       // Cycle through lower positions
#define SPC_LEFT  &kp LA(LG(LEFT))       // Cycle through left positions
#define SPC_RIGHT &kp LA(LG(RIGHT))      // Cycle through right positions
#define SPC_FULL  &kp LA(LG(F))          // Fullscreen
#define SPC_MVR   &kp LC(LA(LG(RIGHT)))  // Move right one screen
#define SPC_MVL   &kp LC(LA(LG(LEFT)))   // Move left one screen
#define SPC_UNDO  &kp LC(LA(LG(Z)))      // Undo last spectacle action

// Mouse Layer command shortcuts
// All keys become tap: command + (key) and long tap: shift + command + (key)
#define Q_F &mt LS(LG(F)) LG(F)
#define Q_P &mt LS(LG(P)) LG(P)
#define Q_B &mt LS(LG(B)) LG(B)
#define Q_G &mt LS(LG(G)) LG(G)
#define Q_Z &mt LS(LG(Z)) LG(Z)
#define Q_X &mt LS(LG(X)) LG(X)
#define Q_C &mt LS(LG(C)) LG(C)
#define Q_D &mt LS(LG(D)) LG(D)
#define Q_V &mt LS(LG(V)) LG(V)

#define LOCK_SCRN   &kp LC(LG(Q))
#define CANCEL      &kp K_CANCEL        // cancel caps-word, num-word and smart-mouse
#define LHYP        LS(LC(LA(LGUI)))
#define RHYP        RS(RC(RA(RGUI)))
#define COPY_CUT    &mt LG(X) LG(C)
#define CMD_PASTE   &kp LG(V)
#define CMD_UNDO    &kp LG(Z)
#define CMD_REDO    &kp LS(LG(Z))
#define PREV_WINDOW &kp LS(LG(GRAVE))
#define NEXT_WINDOW &kp LG(GRAVE)
#define NEXT_TAB    &mtabnext          // next tab in vim or browser(with vimium)
#define PREV_TAB    &mtabprev          // previous tab in vim or browser(with vimium)
#define DSK_PREV    &kp LC(LEFT)       // previous desktop
#define DSK_NEXT    &kp LC(RIGHT)      // next     desktop
#define SWAPPER     &swapper
#define SWAP_PREV   &kp LS(TAB)

// Thumb keys
#define SPC_NAV     &lt_spc NAV 0      // tap: space | shift + tap: underscore | hold: NAV layer
#define RET_HYP     &mt RHYP ENTER     // tap: enter | hold: HYPER key
#define BSPC_SYM    &lt_del SYM 0      // tap: backspace | lshft + tap: delete | rshft + tap: shift-delete | hold: SYM layer
#define SHFT_NUM    &shift_num NUM 0   // tap: sticky shift | double-tap: sticky num layer | hold: NUM layer

/* #define MAKE_LONG_HOLD(NAME, HOLD) \ */
/*     ZMK_BEHAVIOR(NAME, hold_tap, \ */
/*         flavor = "tap-preferred"; \ */
/*         tapping-term-ms = <300>; \ */
/*         quick-tap-ms = <QUICK_TAP_MS>; \ */
/*         bindings = <HOLD, &none>; \ */
/*     ) */
/* MAKE_LONG_HOLD(hbt, &bt) */
// TODO: figure out how to get these things wrapped in a hold tap so that accidental taps don't change the bluetooth profile
#define BT_0 &bt BT_SEL 0
#define BT_1 &bt BT_SEL 1
#define BT_2 &bt BT_SEL 2
#define BT_3 &bt BT_SEL 3
#define BT_4 &bt BT_SEL 4