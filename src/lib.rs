#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

//! FocusCompass cekirdegi: gorev deposu, pomodoro oturum gecmisi, bagimlilik cozumu
//! ve gunluk kapasite planlayicisi.
//!
//! Modul bagimliligi tek yonludur: `hata` hicbir seye bagimli degildir; `gorev`,
//! `oturum`, `kapasite` ve `rapor` yalnizca `hata` ve `zaman` uzerine kurulur;
//! `depo` bu cekirdek tipleri diske yazar; `cli` ise hepsini birbirine baglar.
//! Arayuz katmani (terminal) `cli` icinde, dosya sistemi ise `depo` icinde
//! kalmistir; boylece cekirdek test edilebilir ve tek basina kullanilabilir.
//!
//! Zaman dilimi karmasikligindan kacinmak icin tarih/saat islemleri yalnizca
//! `std::time` uzerine kuruludur (`chrono` ve `time` crate'leri yasaktir) ve
//! kullanici saat dilimi `--tz-offset` bayragi ile dakika cinsinden kabul edilir.

pub mod cli;
pub mod depo;
pub mod gorev;
pub mod hata;
pub mod kapasite;
pub mod oturum;
pub mod rapor;
pub mod zaman;

pub use hata::{Hata, Sonuc};
pub use zaman::SaatDilimi;
