// for test run 'clear && cargo test jsonc'
// ----------------------------------------
// start test => cargo test --lib -- --exact

// Attention pour simplifier l'écriture des test le séparateur '\i' est remplacé par '|'

use crate::prelude::*;

fn parse_test(data: &str) -> Result<TicValue, LinkyError> {
    let text:String = data.chars()
    .map(|x| match x {
        '|' => 0x09 as char,
        _ => x
    }).collect();
    tic_from_str(text.as_str())
}

#[test]
fn parse_meta() {

    parse_test("LTARF|H PLEINE|P\r\n").unwrap();
    parse_test("NGTF|H PLEINE-CREUSE|Z\r\n").unwrap();
    parse_test("VTIC|02|Z\r\n").unwrap();
    parse_test("DATE|H231110100819|Z\r\n").unwrap();
    parse_test("STGE|002A0011|Z\r\n").unwrap(); // register status (doc 6.2.3.14)
    parse_test("ADSC|0123456789012|Z\r\n").unwrap(); // addresse compteur
    parse_test("MSG1|PAS DE MESSAGE|Z\r\n").unwrap();
    parse_test("MSG2|SHORT|Z\r\n").unwrap();
    parse_test("PRM|00000000000000|Z\r\n").unwrap(); // N° PTS distribution
    parse_test("RELAIS|00|Z\r\n").unwrap(); // Position du relay on/off
    parse_test("NTARF|01|Z\r\n").unwrap(); // Index tarrifaire actif
    parse_test("NJOURF|00|Z\r\n").unwrap(); // Numéro du jour en cours calendrier fournisseur
    parse_test("NJOURF+1|00|Z\r\n").unwrap(); // Numéro du jour en cours calendrier fournisseur
    parse_test("PJOURF+1|00000001 16000002 NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE|Z\r\n").unwrap(); // Profil du prochain jour calendrier fournisseur
    parse_test("PPOINTE|00000001 16000002 NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE NONUTILE|Z\r\n").unwrap(); // Profil du prochain jour calendrier fournisseur
}

#[test]
fn parse_depassement() {

    parse_test("ADPS|23|J\r\n").unwrap();   // puissance dépassée A
}


#[test]
fn parse_puissance() {

    parse_test("PREF|22|J\r\n").unwrap();   // puissance préférée kVA
    parse_test("PCOU|22|;\r\n").unwrap();   // puissance de coupure
}

#[test]
fn parse_insts() {

    parse_test("SINSTS|00022|J\r\n").unwrap();
    parse_test("SINSTS1|00022|;\r\n").unwrap();
    parse_test("SINSTS2|00000|8\r\n").unwrap();
    parse_test("SINSTS3|00000|9\r\n").unwrap();
}

#[test]
fn parse_smaxsn() {
    // puissance max soutiré par phase heure/value Wh
    parse_test("SMAXSN|H231109111001|00020|#\r\n").unwrap();
    parse_test("SMAXSN1|H231109111001|00020|T\r\n").unwrap();
    parse_test("SMAXSN2|H231109000000|00000|O\r\n").unwrap();
    parse_test("SMAXSN3|H231109000000|00000|P\r\n").unwrap();
}

#[test]
fn parse_umoy() {
    // tention moyenne V
    parse_test("UMOY1|H231109111001|240|T\r\n").unwrap();
    parse_test("UMOY2|H231109000000|238|O\r\n").unwrap();
    parse_test("UMOY3|H231109000000|239|P\r\n").unwrap();
}


#[test]
fn parse_inject() {
    // Energie active injectée
    parse_test("EAIT|000054878|/\r\n").unwrap();
}

#[test]
fn parse_eait() {
    // puissance max réactive par phase heure/value VArh
    parse_test("EAIT|000038323|#\r\n").unwrap();
    parse_test("EAIT1|000038323|T\r\n").unwrap();
    parse_test("EAIT2|000038323|O\r\n").unwrap();
    parse_test("EAIT3|000038323|P\r\n").unwrap();
}

#[test]
fn parse_irms() {
    // Courant efficace A
    parse_test("IRMS1|000038323|5\r\n").unwrap();
    parse_test("IRMS2|000016555|9\r\n").unwrap();
    parse_test("IRMS3|000000000|$\r\n").unwrap();
}


#[test]
fn parse_urms() {
    // Tension efficace V
    parse_test("URMS1|230|5\r\n").unwrap();
    parse_test("URMS2|231|9\r\n").unwrap();
    parse_test("URMS3|229|$\r\n").unwrap();
}

#[test]
fn parse_mobile() {
    // Debut/Fin point mobile
    parse_test("DPM1|H231109111001|22|5\r\n").unwrap();
    parse_test("FPM1|H231109111001|22|9\r\n").unwrap();
    parse_test("DPM2|H231109111001|22|5\r\n").unwrap();
    parse_test("FPM2|H231109111001|22|9\r\n").unwrap();
    parse_test("DPM3|H231109111001|22|5\r\n").unwrap();
    parse_test("FPM3|H231109111001|22|9\r\n").unwrap();
}

#[test]
fn parse_watt() {
    // Energie active soutirée Fournisseur Wh
    parse_test("EAST|000054878|/\r\n").unwrap();
    parse_test("EASF01|000038323|5\r\n").unwrap();
    parse_test("EASF02|000016555|9\r\n").unwrap();
    parse_test("EASF03|000000000|$\r\n").unwrap();
    parse_test("EASF04|000000000|%\r\n").unwrap();
    parse_test("EASF05|000000000|&\r\n").unwrap();
    parse_test("EASF06|000000000|'\r\n").unwrap();
    parse_test("EASF07|000000000|(\r\n").unwrap();
    parse_test("EASF08|000000000|)\r\n").unwrap();
    parse_test("EASF09|000000000|*\r\n").unwrap();
    parse_test("EASF10|000000000|\"\r\n").unwrap();
}

#[test]
fn parse_misc() {
    // Tension efficace V
    parse_test("CCAIN|H231109111001|00230|5\r\n").unwrap(); // Point n de la courbe de charge active injectée
    parse_test("CCAIN-1|H231109111001|00230|5\r\n").unwrap(); // Point n de la courbe de charge active injectée
    parse_test("URMS2|231|9\r\n").unwrap();
    parse_test("URMS3|229|$\r\n").unwrap();
}
