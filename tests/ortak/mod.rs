//! Entegrasyon testleri icin paylasilan yardimcilar.
//!
//! `tempfile` crate'i bagimlilik politikasiyla yasaktir (WORKER_CONTRACT.md
//! 3.2); gecici dizin yardimcisi kendi kodumuzla yazildi.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Test icinde gecici dizin ureten, `Drop` ile temizleyen kapsayici.
///
/// Benzersizlik etiket + surec kimliginden gelir; rastgelelik crate'i yoktur.
/// Ayni etiketle ikinci kez acildiginda onceki icerik silinir, boylece testler
/// birbirinden bagimsiz olur ve yeniden calistirildiginda kirli durum birikmez.
pub struct GeciciDizin {
    yol: PathBuf,
}

impl GeciciDizin {
    /// `std::env::temp_dir()` altinda etiketten turetilen benzersiz dizin olusturur.
    pub fn yeni(etiket: &str) -> Self {
        let kok = std::env::temp_dir().join(format!("fc-it-{etiket}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&kok);
        std::fs::create_dir_all(&kok).expect("gecici dizin olusturulamadi");
        Self { yol: kok }
    }

    /// Dizin icine goreli yol dondurur.
    pub fn yol(&self) -> &Path {
        &self.yol
    }

    /// Dizin icine yeni bir alt dizin olusturup yolunu dondurur.
    pub fn alt(&self, ad: &str) -> PathBuf {
        let yol = self.yol.join(ad);
        std::fs::create_dir_all(&yol).expect("alt dizin olusturulamadi");
        yol
    }
}

impl Drop for GeciciDizin {
    fn drop(&mut self) {
        // Drop icinden hata dondurulemez; temizlik basarisiz olsa da testi dusurmemeli.
        let _ = std::fs::remove_dir_all(&self.yol);
    }
}

/// Test icin sabit bir saat dilimi (Turkiye, UTC+3).
pub fn tr() -> focuscompass::SaatDilimi {
    focuscompass::SaatDilimi::yeni(180).expect("gecerli saat dilimi")
}

/// `YYYY-W40` icin Pazartesi gun sayisi.
pub fn pazartesi_2026_w40() -> i64 {
    focuscompass::zaman::hafta_ayir("2026-W40").expect("gecerli hafta")
}
