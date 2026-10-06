//! Aachen ibox digit 0 is mouseevent `ibox_taste_D11` (not `…D0`). The frame macro
//! multiplies `ibox_eingabe` by 10 while that flag is set — the same path as digits 1–9.
//!
//! Stock `ibox.osc` stores digits in float `ibox_eingabe` and sets display via
//! `$IntToStr` (or PIN star-mask from float length), then blanks `ibox_Zahleingabe`
//! whenever the float is 0. Middle zeros (1 then 0 → 10) work; a leading 0 alone is a
//! no-op in the float. Compat rewrites PIN (mode 2), line (mode 7) and route (mode 10)
//! into a string buffer on `ibox_Zahleingabe`: empty until typed, append each key
//! (route/line: `0` then `1` → `"01"`; PIN: each key appends `"*"`), never
//! `"02" $IntToStrEnh` auto-pad / prefilled `"00"`.
//!
//! Stock also caps entry at **4 digits** (`ibox_eingabe > 9999` → ÷10). Hof display
//! `03301` (Vaals Flats / trip 3301) is line `33`/`033` then route `01`, not one
//! five-character code. Compat mirrors that with `$length 4 <` on the string buffer.
//!
//! The fixture includes `clickspots_reset` (as on the real AC O530 ibox): it must not wipe
//! the digit buffer every frame in modes 2 / 7 / 10, or digits disappear after the press frame.

use omsi_script::{compile, CompileInput, NullHost, Program, State, Vm};
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/_ibox_d11_fixture")
}

/// Minimal extract of AC O530 `ibox.osc`: digit keys, mode-2/7/10 display, GetRouteIndex
/// packing (compat recognition), and the frame rule that blanks `ibox_Zahleingabe`.
fn aachen_ibox_script() -> String {
    r#"{trigger:ibox_taste_D1}
	1 (S.L.ibox_taste_D1)
{end}
{trigger:ibox_taste_D1_drag}
	0 (S.L.ibox_taste_D1)
{end}
{trigger:ibox_taste_D2}
	1 (S.L.ibox_taste_D2)
{end}
{trigger:ibox_taste_D2_drag}
	0 (S.L.ibox_taste_D2)
{end}
{trigger:ibox_taste_D3}
	1 (S.L.ibox_taste_D3)
{end}
{trigger:ibox_taste_D3_drag}
	0 (S.L.ibox_taste_D3)
{end}
{trigger:ibox_taste_D11}
	1 (S.L.ibox_taste_D11)
{end}
{trigger:ibox_taste_D11_drag}
	0 (S.L.ibox_taste_D11)
{end}
{macro:clickspots_reset}
	"" (S.$.ibox_StatuszeileMitte) (S.$.ibox_StatuszeileRechts) (S.$.ibox_Zahleingabe) (S.$.ibox_delay) (S.$.ibox_hstverlauf)
{end}
{macro:ibox_tastatur}
(L.L.ibox_taste_D1) 1 =
{if}
	(L.L.ibox_eingabe) 10 * trunc (S.L.ibox_eingabe)
	(L.L.ibox_eingabe) 1 + trunc (S.L.ibox_eingabe)
{endif}
(L.L.ibox_taste_D2) 1 =
{if}
	(L.L.ibox_eingabe) 10 * trunc (S.L.ibox_eingabe)
	(L.L.ibox_eingabe) 2 + trunc (S.L.ibox_eingabe)
{endif}
(L.L.ibox_taste_D3) 1 =
{if}
	(L.L.ibox_eingabe) 10 * trunc (S.L.ibox_eingabe)
	(L.L.ibox_eingabe) 3 + trunc (S.L.ibox_eingabe)
{endif}
(L.L.ibox_taste_D11) 1 =
{if}
	(L.L.ibox_eingabe) 10 * trunc (S.L.ibox_eingabe)
{endif}
(L.L.ibox_taste_D10) 1 =
{if}
	(L.L.ibox_eingabe) 10 / trunc (S.L.ibox_eingabe)
	0 (S.L.ibox_taste_D10)
{endif}
'4 Ziffern Begrenzung
(L.L.ibox_eingabe) 9999 >
{if}
	(L.L.ibox_eingabe) 10 / trunc (S.L.ibox_eingabe)
{endif}
{end}
{macro:ibox_route_lookup}
	(L.L.ibox_eingabe) s4 (S.L.ibox_Route)
	(L.L.IBIS_LinieKurs) 100 * l4 + (M.V.GetRouteIndex) (S.L.ibox_routenindex)
{end}
{frame}
	(M.L.clickspots_reset)
	(L.L.ibox_modus) 2 =
	{if}
		(M.L.ibox_tastatur)
		(L.L.ibox_eingabe) $IntToStr $length s0
		"*" l0 $* (S.$.ibox_Zahleingabe)
	{endif}
	(L.L.ibox_modus) 7 =
	{if}
		(M.L.ibox_tastatur)
		(L.L.ibox_eingabe) $IntToStr (S.$.ibox_Zahleingabe)
	{endif}
	(L.L.ibox_modus) 10 =
	{if}
		(M.L.ibox_tastatur)
		(L.L.ibox_eingabe) (S.L.ibox_Route)
		(L.L.ibox_eingabe) $IntToStr (S.$.ibox_Zahleingabe)
	{endif}
	(L.L.ibox_eingabe) 0 =
	{if}
		"" (S.$.ibox_Zahleingabe)
	{endif}
{end}
"#
    .to_string()
}

fn compile_fixture() -> Program {
    let dir = fixture_dir();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("ibox.osc"), aachen_ibox_script()).unwrap();
    std::fs::write(
        dir.join("v.txt"),
        "ibox_taste_D1\nibox_taste_D2\nibox_taste_D3\nibox_taste_D4\nibox_taste_D5\n\
ibox_taste_D6\nibox_taste_D7\nibox_taste_D8\nibox_taste_D9\n\
ibox_taste_D10\nibox_taste_D11\nibox_eingabe\nibox_modus\nibox_Route\nIBIS_LinieKurs\nibox_routenindex\nibox_PIN\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("s.txt"),
        "ibox_Zahleingabe\nibox_StatuszeileMitte\nibox_StatuszeileRechts\nibox_delay\nibox_hstverlauf\n",
    )
    .unwrap();

    let p = compile(&CompileInput {
        varlists: vec![dir.join("v.txt")],
        stringvarlists: vec![dir.join("s.txt")],
        scripts: vec![dir.join("ibox.osc")],
        ..Default::default()
    });
    assert!(p.errors.is_empty(), "{:?}", p.errors);
    p
}

#[test]
fn d11_digit_zero_appends_like_other_digits() {
    let p = compile_fixture();
    let d11 = p.var("ibox_taste_D11").unwrap();
    let e = p.var("ibox_eingabe").unwrap();
    let modus = p.var("ibox_modus").unwrap();
    let z = p.str_var("ibox_Zahleingabe").unwrap();
    let mut vm = Vm::new();
    let mut st = State::new(&p);
    let mut host = NullHost;

    st.set(modus, 7.0);

    // Mid zero: type 1 then 0 → 10
    st.set(e, 1.0);
    st.str_vars[z as usize] = "1".into();
    assert!(vm.run_trigger(&p, "ibox_taste_D11", &mut st, &mut host));
    assert_eq!(st.get(d11), 1.0, "press must set ibox_taste_D11");
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(st.get(e), 10.0, "digit 0 must turn 1 into 10");
    assert_eq!(st.str_vars[z as usize], "10", "display must show the middle zero");

    assert!(vm.run_trigger(&p, "ibox_taste_D11_drag", &mut st, &mut host));
    assert_eq!(st.get(d11), 0.0);
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(st.get(e), 10.0, "cleared flag must not multiply again");
    assert_eq!(st.str_vars[z as usize], "10");
}

#[test]
fn d11_after_d1_shows_ten_like_click_path() {
    let p = compile_fixture();
    let e = p.var("ibox_eingabe").unwrap();
    let modus = p.var("ibox_modus").unwrap();
    let z = p.str_var("ibox_Zahleingabe").unwrap();
    let mut vm = Vm::new();
    let mut st = State::new(&p);
    let mut host = NullHost;
    st.set(modus, 7.0);
    st.set(e, 0.0);

    assert!(vm.run_trigger(&p, "ibox_taste_D1", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D1_drag", &mut st, &mut host));
    assert_eq!(st.get(e), 1.0);
    assert_eq!(st.str_vars[z as usize], "1");

    assert!(vm.run_trigger(&p, "ibox_taste_D11", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D11_drag", &mut st, &mut host));
    assert_eq!(st.get(e), 10.0);
    assert_eq!(st.str_vars[z as usize], "10");
}

#[test]
fn mode10_leading_zero_then_one_shows_route_01() {
    let p = compile_fixture();
    let e = p.var("ibox_eingabe").unwrap();
    let modus = p.var("ibox_modus").unwrap();
    let route = p.var("ibox_Route").unwrap();
    let z = p.str_var("ibox_Zahleingabe").unwrap();
    let mut vm = Vm::new();
    let mut st = State::new(&p);
    let mut host = NullHost;

    st.set(modus, 10.0);
    st.set(e, 0.0);

    // Route entry starts empty — nothing prefilled before the driver types.
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(st.str_vars[z as usize], "", "no prefilled 00");

    // Press 0 (D11): float stays 0, display shows the typed leading zero.
    assert!(vm.run_trigger(&p, "ibox_taste_D11", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D11_drag", &mut st, &mut host));
    assert_eq!(st.get(e), 0.0);
    assert_eq!(st.str_vars[z as usize], "0");
    // clickspots_reset runs every frame — digit must survive idle frames after release.
    vm.run_frame(&p, &mut st, &mut host);
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(
        st.str_vars[z as usize], "0",
        "leading zero must survive clickspots_reset (not only the press frame)"
    );

    // Press 1: float 1, display "01" — hof route 01 / GetRouteIndex(line×100+1).
    assert!(vm.run_trigger(&p, "ibox_taste_D1", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D1_drag", &mut st, &mut host));
    assert_eq!(st.get(e), 1.0);
    assert_eq!(st.str_vars[z as usize], "01");
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(st.str_vars[z as usize], "01", "01 must persist across frames");
    assert_eq!(st.get(route), 1.0, "submitted route number is 1 (shown as 01)");

    // Typing only 1 (no leading 0 key) shows "1", not a padded 01.
    st.set(e, 0.0);
    st.str_vars[z as usize].clear();
    assert!(vm.run_trigger(&p, "ibox_taste_D1", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D1_drag", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(st.str_vars[z as usize], "1");
    assert_ne!(st.str_vars[z as usize], "01", "must not auto-pad without typing 0");
}

#[test]
fn mode7_leading_zero_then_one_shows_line_01() {
    let p = compile_fixture();
    let e = p.var("ibox_eingabe").unwrap();
    let modus = p.var("ibox_modus").unwrap();
    let z = p.str_var("ibox_Zahleingabe").unwrap();
    let mut vm = Vm::new();
    let mut st = State::new(&p);
    let mut host = NullHost;

    st.set(modus, 7.0);
    st.set(e, 0.0);
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(st.str_vars[z as usize], "", "line entry starts empty");

    assert!(vm.run_trigger(&p, "ibox_taste_D11", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D11_drag", &mut st, &mut host));
    assert_eq!(st.get(e), 0.0);
    assert_eq!(st.str_vars[z as usize], "0");
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(
        st.str_vars[z as usize], "0",
        "line leading zero must survive clickspots_reset"
    );

    assert!(vm.run_trigger(&p, "ibox_taste_D1", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D1_drag", &mut st, &mut host));
    assert_eq!(st.get(e), 1.0);
    assert_eq!(st.str_vars[z as usize], "01");
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(st.str_vars[z as usize], "01");
}

#[test]
fn mode2_pin_leading_zero_appends_star() {
    let p = compile_fixture();
    let e = p.var("ibox_eingabe").unwrap();
    let modus = p.var("ibox_modus").unwrap();
    let z = p.str_var("ibox_Zahleingabe").unwrap();
    let mut vm = Vm::new();
    let mut st = State::new(&p);
    let mut host = NullHost;

    st.set(modus, 2.0);
    st.set(e, 0.0);
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(st.str_vars[z as usize], "", "PIN entry starts empty");

    // Leading 0: float stays 0, one masked star.
    assert!(vm.run_trigger(&p, "ibox_taste_D11", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D11_drag", &mut st, &mut host));
    assert_eq!(st.get(e), 0.0);
    assert_eq!(st.str_vars[z as usize], "*");
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(
        st.str_vars[z as usize], "*",
        "PIN star must survive clickspots_reset"
    );

    assert!(vm.run_trigger(&p, "ibox_taste_D1", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D1_drag", &mut st, &mut host));
    assert_eq!(st.get(e), 1.0);
    assert_eq!(st.str_vars[z as usize], "**");
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(st.str_vars[z as usize], "**");
}

#[test]
fn mode7_line_entry_starts_empty_after_route_display() {
    let p = compile_fixture();
    let e = p.var("ibox_eingabe").unwrap();
    let modus = p.var("ibox_modus").unwrap();
    let z = p.str_var("ibox_Zahleingabe").unwrap();
    let mut vm = Vm::new();
    let mut st = State::new(&p);
    let mut host = NullHost;

    // Route mode with a typed leading zero on screen…
    st.set(modus, 10.0);
    st.set(e, 0.0);
    assert!(vm.run_trigger(&p, "ibox_taste_D11", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D11_drag", &mut st, &mut host));
    assert_eq!(st.str_vars[z as usize], "0");

    // …then line entry: field must be empty so the driver types the line themselves.
    st.set(modus, 7.0);
    st.set(e, 0.0);
    st.str_vars[z as usize].clear();
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(st.str_vars[z as usize], "");
}

#[test]
fn mode10_route_02_and_mid_zero_ten() {
    let p = compile_fixture();
    let e = p.var("ibox_eingabe").unwrap();
    let modus = p.var("ibox_modus").unwrap();
    let z = p.str_var("ibox_Zahleingabe").unwrap();
    let mut vm = Vm::new();
    let mut st = State::new(&p);
    let mut host = NullHost;
    st.set(modus, 10.0);

    st.set(e, 0.0);
    assert!(vm.run_trigger(&p, "ibox_taste_D11", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D11_drag", &mut st, &mut host));
    assert_eq!(st.str_vars[z as usize], "0");

    // Digit 2 typed after leading zero → "02" (not IntToStrEnh of float alone).
    assert!(vm.run_trigger(&p, "ibox_taste_D2", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D2_drag", &mut st, &mut host));
    assert_eq!(st.get(e), 2.0);
    assert_eq!(st.str_vars[z as usize], "02");

    st.set(e, 1.0);
    st.str_vars[z as usize] = "1".into();
    assert!(vm.run_trigger(&p, "ibox_taste_D11", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert_eq!(st.get(e), 10.0);
    assert_eq!(st.str_vars[z as usize], "10");
}

/// Hof trip 3301 (Vaals Flats) shows as header `03301` = padded line `033` + route `01`.
/// Type those in two fields — not one five-digit string (stock max 4 digits / chars).
#[test]
fn mode7_line_033_then_mode10_route_01_for_hof_3301() {
    let p = compile_fixture();
    let e = p.var("ibox_eingabe").unwrap();
    let modus = p.var("ibox_modus").unwrap();
    let z = p.str_var("ibox_Zahleingabe").unwrap();
    let mut vm = Vm::new();
    let mut st = State::new(&p);
    let mut host = NullHost;

    // Linie: 0-3-3 → "033" (IBIS_LinieKurs = 33 after Enter in the real script).
    st.set(modus, 7.0);
    st.set(e, 0.0);
    vm.run_frame(&p, &mut st, &mut host);
    for (trig, drag) in [
        ("ibox_taste_D11", "ibox_taste_D11_drag"),
        ("ibox_taste_D3", "ibox_taste_D3_drag"),
        ("ibox_taste_D3", "ibox_taste_D3_drag"),
    ] {
        assert!(vm.run_trigger(&p, trig, &mut st, &mut host));
        vm.run_frame(&p, &mut st, &mut host);
        assert!(vm.run_trigger(&p, drag, &mut st, &mut host));
    }
    assert_eq!(st.get(e), 33.0);
    assert_eq!(st.str_vars[z as usize], "033");

    // Route: 0-1 → "01" → GetRouteIndex(33×100+1) = 3301 in the real script.
    st.set(modus, 10.0);
    st.set(e, 0.0);
    st.str_vars[z as usize].clear();
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D11", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D11_drag", &mut st, &mut host));
    assert!(vm.run_trigger(&p, "ibox_taste_D1", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D1_drag", &mut st, &mut host));
    assert_eq!(st.get(e), 1.0);
    assert_eq!(st.str_vars[z as usize], "01");
}

#[test]
fn stock_4_digit_cap_rejects_fifth_char_and_fifth_significant_digit() {
    let p = compile_fixture();
    let e = p.var("ibox_eingabe").unwrap();
    let modus = p.var("ibox_modus").unwrap();
    let z = p.str_var("ibox_Zahleingabe").unwrap();
    let mut vm = Vm::new();
    let mut st = State::new(&p);
    let mut host = NullHost;

    st.set(modus, 7.0);
    st.set(e, 0.0);
    // Type 3301 (4 digits) — allowed.
    for (trig, drag) in [
        ("ibox_taste_D3", "ibox_taste_D3_drag"),
        ("ibox_taste_D3", "ibox_taste_D3_drag"),
        ("ibox_taste_D11", "ibox_taste_D11_drag"),
        ("ibox_taste_D1", "ibox_taste_D1_drag"),
    ] {
        assert!(vm.run_trigger(&p, trig, &mut st, &mut host));
        vm.run_frame(&p, &mut st, &mut host);
        assert!(vm.run_trigger(&p, drag, &mut st, &mut host));
    }
    assert_eq!(st.get(e), 3301.0);
    assert_eq!(st.str_vars[z as usize], "3301");

    // Fifth digit: stock float ÷10, string stays at 4 chars.
    assert!(vm.run_trigger(&p, "ibox_taste_D2", &mut st, &mut host));
    vm.run_frame(&p, &mut st, &mut host);
    assert!(vm.run_trigger(&p, "ibox_taste_D2_drag", &mut st, &mut host));
    assert_eq!(st.get(e), 3301.0, "stock 9999 cap must drop the 5th digit");
    assert_eq!(
        st.str_vars[z as usize], "3301",
        "string buffer must not grow past 4 characters"
    );
}
