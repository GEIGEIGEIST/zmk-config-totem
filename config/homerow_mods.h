/* Homerow mods */
#define KEYS_L LT0 LT1 LT2 LT3 LT4 LM0 LM1 LM2 LM3 LM4 LB0 LB1 LB2 LB3 LB4  // left hand
#define KEYS_R RT0 RT1 RT2 RT3 RT4 RM0 RM1 RM2 RM3 RM4 RB0 RB1 RB2 RB3 RB4  // right hand
#define THUMBS LH2 LH1 LH0 RH0 RH1 RH2                                     // thumbs

#define MAKE_HRM(NAME, HOLD, TAP, TRIGGER_POS) \
  ZMK_BEHAVIOR(NAME, hold_tap, \
      flavor = "balanced"; \
      tapping-term-ms = <280>; \
      quick-tap-ms = <QUICK_TAP_MS>; \
      bindings = <HOLD>, <TAP>; \
      hold-trigger-key-positions = <TRIGGER_POS>; \
      hold-trigger-on-release; \
      )

MAKE_HRM(hml, &kp, &kp, KEYS_R THUMBS)  // left-hand HRMs
MAKE_HRM(hmr, &kp, &kp, KEYS_L THUMBS)  // right-hand

// Base Layer home row mods
#define A_LGUI     &hml LGUI A
#define R_LALT     &hml LALT R
#define S_LCTRL    &hml LCTRL S
#define T_LSHFT    &hml LSHFT T
#define N_RSHFT    &hmr RSHFT N
#define E_RCTRL    &hmr RCTRL E
#define I_RALT     &hmr RALT I
#define O_RGUI     &hmr RGUI O


// SYM Layer home row mods
#define SYM_CARET  &hml LGUI CARET
#define SYM_MINUS  &hml LALT MINUS
#define SYM_PLUS   &hml LCTRL PLUS
#define SYM_EQUAL  &hml LSHFT EQUAL
#define SYM_LBKT   &hmr LSHFT LBKT
#define SYM_RBKT   &hmr RCTRL RBKT
#define SYM_AT     &hmr RALT AT
#define SYM_DOLLAR &hmr RGUI DOLLAR