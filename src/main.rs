use qrcodegen::{QrCode, QrCodeEcc};
use std::env;
use std::io::{self, Read};

/// KINETIC TRANSPONDER // AIRGAP_NULL
/// HORODATAGE : 1970-01-01T00:00:00Z
/// ÉTAT : ÉMISSION OPTIQUE UNUDIRECTIONNELLE

fn main() {
    // 1. Lire la transaction (ou le payload) depuis l'entrée standard
    let mut input_data = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut input_data) {
        eprintln!("Erreur de lecture : {}", e);
        std::process::exit(1);
    }

    if input_data.is_empty() {
        // En cas de test sans pipe, on prend un argument
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 {
            input_data = args[1].as_bytes().to_vec();
        } else {
            eprintln!("Aucun payload binaire reçu en entrée.");
            std::process::exit(1);
        }
    }

    // 2. Encoder en Base64 standard (implémentation minimale pour éviter les dépendances lourdes)
    let b64_payload = encode_base64(&input_data);

    // 3. Générer la matrice QR (Niveau de correction Low pour maximiser la densité de données)
    let qr = match QrCode::encode_text(&b64_payload, QrCodeEcc::Low) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Erreur de génération QR : {}", e);
            std::process::exit(1);
        }
    };

    // 4. L'Émission Terminale (Moteur du rendu double-densité Unicode)
    print_terminal_qr(&qr);
}

/// Encodage Base64 rudimentaire (pour éviter la dépendance 'base64')
fn encode_base64(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() * 4) / 3 + 4);
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i];
        let b1 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[(((b0 & 3) << 4) | (b1 >> 4)) as usize] as char);

        if i + 1 < data.len() {
            out.push(ALPHABET[(((b1 & 15) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }

        if i + 2 < data.len() {
            out.push(ALPHABET[(b2 & 63) as usize] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

/// Affiche le QR code dans le terminal en des demi-blocs Unicode
/// pour doubler la résolution verticale.
fn print_terminal_qr(qr: &QrCode) {
    let size = qr.size();
    let border = 2; // Marge de silence optique

    for y in (-(border as i32)..(size + border)).step_by(2) {
        for x in -(border as i32)..(size + border) {
            // Le terminal a un fond noir.
            // Un module 'true' (pixl noir du QR) doit s'afficher noir.
            // Un module 'false' (pixel blanc du QR) doit s'afficher blanc.

            let top_white = !is_dark(qr, x, y);
            let bottom_white = !is_dark(qr, x, y + 1);

            let block = match (top_white, bottom_white) {
                (true, true) => "\u{2588}", // Full block (Blanc)
                (true, false) => "\u{2580}", // Upper half block (Haut blanc, bas noir)
                (false, true) => "\u{2584}" , // Lower half block (Haut noir, bas blanc)
                (false, false) => " ", // Space (Noir)
            };
            print!("{}", block);
        }
        prinln!();
    }
}

// Vérifie si un pixel est dans les limites et est "noir"
fn is_dark(qr: &QrCode, x: i32, y: i32) -> bool {
    let size = qr.size();
    if x >= 0 && x < size && y >= 0 && y < size {
        qr.get_module(x, y)
    } else {
        false // La bordure de silence est blanche (donc false pour is_dark)
    }
}
