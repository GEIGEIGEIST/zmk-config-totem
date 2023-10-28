/*
 * helper.h
 *
 * Convenience macros simplifying ZMK's keymap configuration.
 * See https://github.com/urob/zmk-nodefree-config for documentation.
 */

#pragma once

#define ZMK_HELPER_STRINGIFY(x) #x

/* ZMK_BEHAVIOR */

#define ZMK_BEHAVIOR_CORE_caps_word   compatible = "zmk,behavior-caps-word";  #binding-cells = <0>
#define ZMK_BEHAVIOR_CORE_hold_tap    compatible = "zmk,behavior-hold-tap";   #binding-cells = <2>
#define ZMK_BEHAVIOR_CORE_key_repeat  compatible = "zmk,behavior-key-repeat"; #binding-cells = <0>
#define ZMK_BEHAVIOR_CORE_macro       compatible = "zmk,behavior-macro";      #binding-cells = <0>
#define ZMK_BEHAVIOR_CORE_mod_morph   compatible = "zmk,behavior-mod-morph";  #binding-cells = <0>
#define ZMK_BEHAVIOR_CORE_sticky_key  compatible = "zmk,behavior-sticky-key"; #binding-cells = <1>
#define ZMK_BEHAVIOR_CORE_tap_dance   compatible = "zmk,behavior-tap-dance";  #binding-cells = <0>
#define ZMK_BEHAVIOR_CORE_tri_state   compatible = "zmk,behavior-tri-state";  #binding-cells = <0>

#define ZMK_BEHAVIOR(name, type, ...) \
    / { \
        behaviors { \
            name: name { \
                label = ZMK_HELPER_STRINGIFY(ZB_ ## name); \
                ZMK_BEHAVIOR_CORE_ ## type; \
                __VA_ARGS__ \
            }; \
        }; \
    };
