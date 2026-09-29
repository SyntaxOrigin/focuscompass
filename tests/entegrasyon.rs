//! Uctan uca entegrasyon testleri: komut satiri -> dosya -> yeniden okuma.
//!
//! Bu dosya tek bir "gunun tam akisini" disa danimlarak dener: gorev ekle,
//! bagimlilik kur, plan cikar, pomodoro baslat, kesinti isaretle, durdur ve
//! haftalik rapor yaz. Amac cekirdek modullerin birbirine nasil baglandigini
//! ve verinin gercekten dosyaya gidip geldigini dogrulamaktir.

mod ortak;

use focuscompass::cli::{
    calistir, AgacArg, BagimlilikCiftArg, BagimlilikKomut, BaslatArg, Cli, DonguArg, DurdurArg,
    EkleArg, KesintiArg, Komut, ListeArg, PlanArg, RaporArg, TurSecimi,
};

use ortak::{pazartesi_2026_w40, tr, GeciciDizin};

fn cli(kok: &std::path::Path, komut: Komut) -> Cli {
    Cli {
        veri: Some(kok.to_path_buf()),
        tz_offset: 180,
        komut,
    }
}

fn ekle(baslik: &str, dakika: i32, oncelik: &str, etiketler: Vec<String>) -> Komut {
    Komut::Add(EkleArg {
        basliklar: vec![baslik.to_string()],
        dakika,
        oncelik: oncelik.to_string(),
        etiket: etiketler,
        bagimlilik: Vec::new(),
    })
}

#[test]
fn gunun_tam_akisi_calisir() {
    let gecici = GeciciDizin::yeni("tam-akis");
    let veri = gecici.alt("veri");

    // 1) Dort gorev eklenir; biri baskasina baglanir.
    calistir(&cli(
        &veri,
        ekle("Musteri onayi", 30, "yuksek", vec!["mürekkep".into()]),
    ))
    .unwrap();
    calistir(&cli(
        &veri,
        ekle("Yayin oncesi duzeltme", 45, "yuksek", vec![]),
    ))
    .unwrap();
    calistir(&cli(
        &veri,
        ekle("Rapor taslagi", 90, "normal", vec!["rapor".into()]),
    ))
    .unwrap();
    calistir(&cli(&veri, ekle("Arsiv", 600, "dusuk", vec![]))).unwrap();
    let cikti = calistir(&cli(
        &veri,
        Komut::Dep(BagimlilikKomut::Ekle(BagimlilikCiftArg {
            gorev: 2,
            bagimlilik: 1,
        })),
    ))
    .unwrap();
    assert!(cikti.contains("bagimlilik eklendi"), "{cikti}");

    // 2) Gorev dosyasi gercekten olusturuldu ve surum alani iceriyor.
    let depo = focuscompass::depo::Depo::yeni(&veri);
    let metin = std::fs::read_to_string(depo.gorev_yolu()).expect("gorev dosyasi");
    assert!(metin.contains("\"surum\": 1"), "{metin}");

    // 3) Kapasite plani: 120 dk kapasite, 45 dk sabit sure = 75 dk kullanilabilir.
    let cikti = calistir(&cli(
        &veri,
        Komut::Plan(PlanArg {
            kapasite: 120,
            sabit: 45,
            tarih: Some("2026-09-29".to_string()),
        }),
    ))
    .unwrap();
    assert!(cikti.contains("kullanilabilir 75 dk"), "{cikti}");
    // Yalnizca 30 dk'lik "Musteri onayi" sigar; bagimlilik ve kapasite engelleri vardir.
    assert!(cikti.contains("BUGUN SIGANLAR: 1 gorev, 30 dk"), "{cikti}");
    assert!(cikti.contains("BUGUN SIGMAYANLAR: 3 gorev"), "{cikti}");
    assert!(cikti.contains("[bagimlilik]"), "{cikti}");
    assert!(
        cikti.contains("tamamlanmamis bagimlilik: #1 (bekliyor)"),
        "{cikti}"
    );
    assert!(cikti.contains("[kapasite]"), "{cikti}");
    assert!(cikti.contains("kalan 45 dk yetmiyor"), "{cikti}");
    assert!(cikti.contains("kapasite asimi"), "{cikti}");

    // 4) Bagimlilik nedeniyle baslatilamayan gorev reddedilir.
    let sonuc = calistir(&cli(
        &veri,
        Komut::Start(BaslatArg {
            id: 2,
            dakika: 25,
            tur: TurSecimi::Odak,
        }),
    ));
    assert!(sonuc.is_err(), "bagimlilik varken baslatilmamali");

    // 5) Bagimlilik zincirinin basindan pomodoro dongusu calisir (kuru kip).
    let cikti = calistir(&cli(
        &veri,
        Komut::Cycle(DonguArg {
            id: 1,
            odak_dakika: 25,
            mola_dakika: 5,
            dongu: 1,
            mola_tipi: TurSecimi::Kisa,
            kesinti_dakika: Some(3),
            kesinti_kaynak: "toplanti".to_string(),
            kuru: true,
        }),
    ))
    .unwrap();
    assert!(cikti.contains("dongu 1/1 odak"), "{cikti}");
    assert!(cikti.contains("durum=tamamlandi"), "{cikti}");

    // 6) Artik bagimlilik tamamlandigi icin duzeltme gorevi planlanir.
    let cikti = calistir(&cli(
        &veri,
        Komut::Plan(PlanArg {
            kapasite: 120,
            sabit: 45,
            tarih: Some("2026-09-29".to_string()),
        }),
    ))
    .unwrap();
    assert!(cikti.contains("BUGUN SIGANLAR: 1 gorev, 45 dk"), "{cikti}");
    assert!(cikti.contains("BUGUN SIGMAYANLAR: 2 gorev"), "{cikti}");
    assert!(!cikti.contains("[bagimlilik]"), "engel kalkmali: {cikti}");

    // 7) Yeniden yuklemede veri kaybi yok.
    let yeniden = depo.gorevleri_oku().expect("gorevler okunamadi");
    assert_eq!(yeniden.uzunluk(), 4);
    assert_eq!(
        yeniden.getir(1).unwrap().durum,
        focuscompass::gorev::Durum::Tamamlandi
    );
    assert_eq!(yeniden.getir(2).unwrap().bagimliliklar, vec![1]);
}

#[test]
fn ust_uste_pomodoro_ve_kesinti_gunlugu_akisi() {
    let gecici = GeciciDizin::yeni("kesinti");
    let veri = gecici.alt("veri");
    calistir(&cli(&veri, ekle("Yaz", 25, "normal", vec![]))).unwrap();

    let baslangic = focuscompass::zaman::simdi_epoch();
    calistir(&cli(
        &veri,
        Komut::Start(BaslatArg {
            id: 1,
            dakika: 25,
            tur: TurSecimi::Odak,
        }),
    ))
    .unwrap();

    // Ikinci oturum ust uste acilamaz.
    let sonuc = calistir(&cli(
        &veri,
        Komut::Start(BaslatArg {
            id: 1,
            dakika: 25,
            tur: TurSecimi::Odak,
        }),
    ));
    assert!(sonuc.is_err(), "ust uste pomodoro reddedilmeli");

    // Kesinti kaydi: 12. dakikada 4 dakikalik toplanti.
    let cikti = calistir(&cli(
        &veri,
        Komut::Interrupt(KesintiArg {
            kaynak: "toplanti".to_string(),
            dakika: 4,
            simdi: Some(baslangic + 12 * 60),
        }),
    ))
    .unwrap();
    assert!(cikti.contains("toplam=4 dk"), "{cikti}");

    // Kesinti, oturumdan gecen sureyi asiyorsa reddedilir.
    let sonuc = calistir(&cli(
        &veri,
        Komut::Interrupt(KesintiArg {
            kaynak: "toplanti".to_string(),
            dakika: 20,
            simdi: Some(baslangic + 12 * 60),
        }),
    ));
    assert!(sonuc.is_err(), "gecen sureyi asan kesinti reddedilmeli");

    // Oturum kapatilir; odak suresi kesintiden dusulur.
    let cikti = calistir(&cli(
        &veri,
        Komut::Stop(DurdurArg {
            ertele: false,
            bitis: Some(baslangic + 25 * 60),
        }),
    ))
    .unwrap();
    assert!(cikti.contains("oturum #1 kapandi"), "{cikti}");
    assert!(cikti.contains("21 dk"), "odak 25-4=21 olmali: {cikti}");

    // Ayni sira icin son kayit gecerlidir: gunluk artik kapali.
    let gunluk = focuscompass::oturum::OturumGunlugu::yukle(
        focuscompass::depo::Depo::yeni(&veri)
            .oturumlari_oku()
            .unwrap(),
    );
    assert_eq!(gunluk.uzunluk(), 1);
    assert!(gunluk.acik_oturum().is_none());
    assert_eq!(gunluk.getir(1).unwrap().toplam_kesinti(), 4);
    assert_eq!(gunluk.getir(1).unwrap().odak_dakika().unwrap(), 21);

    // Gunluk Markdown dosyasi yazildi mi?
    let tarih = focuscompass::zaman::tarih_yaz(tr().gun_no(baslangic));
    let gunluk_dosya = focuscompass::depo::Depo::yeni(&veri)
        .gunluk_oku(&tarih)
        .unwrap();
    assert!(
        gunluk_dosya.starts_with("# FocusCompass gunlugu"),
        "{gunluk_dosya}"
    );
    assert!(gunluk_dosya.contains("odak"), "{gunluk_dosya}");
}

#[test]
fn haftalik_rapor_json_ve_markdown_dosyalari_uretir() {
    let gecici = GeciciDizin::yeni("rapor");
    let veri = gecici.alt("veri");
    let pzt = pazartesi_2026_w40();

    // Dort oturum yazilir: Pazartesi 09:00, Pazartesi 09:40, Carsamba 14:00
    // ve Persembe 10:00 (sonuncusu bir sonraki ISO haftasinda kalir).
    let depo = focuscompass::depo::Depo::yeni(&veri);
    depo.hazirla().unwrap();
    const GUN: i64 = 86_400;
    let pazartesi_epoch = pzt * GUN;
    for (index, (gun_kaymasi, saat_dk, planli)) in [
        (0_i64, 9 * 60, 25_i32),
        (0, 9 * 60 + 40, 25),
        (2, 14 * 60, 50),
        (7, 10 * 60, 25),
    ]
    .into_iter()
    .enumerate()
    {
        let mut oturum = focuscompass::oturum::Oturum::yeni(
            index as u32 + 1,
            Some(1),
            focuscompass::oturum::OturumTuru::Odak,
            planli,
            pazartesi_epoch + gun_kaymasi * GUN + saat_dk * 60,
        )
        .unwrap();
        oturum.kesinti_ekle("toplanti", 2, planli).unwrap();
        oturum
            .bitir(oturum.baslangic + i64::from(planli) * 60)
            .unwrap();
        depo.oturum_ekle(&oturum).unwrap();
    }

    let cikti = calistir(&cli(
        &veri,
        Komut::Report(RaporArg {
            hafta: Some("2026-W40".to_string()),
            format: "ikisi".to_string(),
            cikti: Some(veri.clone()),
        }),
    ))
    .unwrap();
    assert!(
        cikti.contains("2026-W40 (2026-09-28 .. 2026-10-04)"),
        "{cikti}"
    );
    assert!(cikti.contains("3 oturum"), "{cikti}");
    assert!(cikti.contains("1 sa 34 dk odak"), "{cikti}");

    // JSON raporu yapilandirilmis olarak geri yuklenebilir.
    let json_yol = veri.join("raporlar").join("hafta-2026-W40.json");
    let metin = std::fs::read_to_string(&json_yol).expect("json raporu");
    let rapor: focuscompass::rapor::HaftaRaporu =
        serde_json::from_str(&metin).expect("gecerli json");
    assert_eq!(rapor.gunler.len(), 7);
    assert_eq!(rapor.toplam_oturum, 3);
    assert_eq!(rapor.toplam_kesinti_dakika, 6);
    assert_eq!(rapor.toplam_odak_dakika, 94);

    // Markdown raporunda tahmin ve gerceklesen yan yana.
    let md_yol = veri.join("raporlar").join("hafta-2026-W40.md");
    let md = std::fs::read_to_string(&md_yol).expect("markdown raporu");
    assert!(md.contains("- Tahmin (tamamlananlar):"), "{md}");
    assert!(md.contains("## Kesinti kaynaklari"), "{md}");
    assert!(md.contains("| toplanti | 6 dk |"), "{md}");
}

#[test]
fn bozuk_dosya_araci_kilitlemez_ve_silinmez() {
    let gecici = GeciciDizin::yeni("bozuk");
    let veri = gecici.alt("veri");
    calistir(&cli(&veri, ekle("Gorev", 30, "normal", vec![]))).unwrap();
    let depo = focuscompass::depo::Depo::yeni(&veri);
    let orijinal = std::fs::read_to_string(depo.gorev_yolu()).unwrap();
    std::fs::write(depo.gorev_yolu(), "{\"surum\":1,\"gorevler\":[ {").unwrap();

    // Bozuk dosya reddedilir, arac calismaya devam eder, veri **silinmez**.
    let sonuc = calistir(&cli(
        &veri,
        Komut::List(ListeArg {
            durum: None,
            etiket: None,
            ozet: true,
        }),
    ));
    assert!(sonuc.is_err(), "bozuk dosya reddedilmeli");
    let bozuk = std::fs::read_to_string(depo.gorev_yolu()).unwrap();
    assert_eq!(
        bozuk, "{\"surum\":1,\"gorevler\":[ {",
        "dosya degistirilmemeli"
    );

    // Kullanici dosyayi duzeltince arac toparlanir.
    std::fs::write(depo.gorev_yolu(), orijinal).unwrap();
    let cikti = calistir(&cli(
        &veri,
        Komut::List(ListeArg {
            durum: None,
            etiket: None,
            ozet: true,
        }),
    ))
    .unwrap();
    assert!(cikti.contains("Gorev"), "{cikti}");
}

#[test]
fn surum_uyusmazligi_ve_asiili_bagimlilik_yonetimi() {
    let gecici = GeciciDizin::yeni("surum");
    let veri = gecici.alt("veri");
    let depo = focuscompass::depo::Depo::yeni(&veri);
    depo.hazirla().unwrap();
    std::fs::write(depo.gorev_yolu(), "{\"surum\":7,\"gorevler\":[]}").unwrap();
    let sonuc = calistir(&cli(
        &veri,
        Komut::List(ListeArg {
            durum: None,
            etiket: None,
            ozet: true,
        }),
    ));
    assert!(sonuc.is_err(), "surum uyusmazligi reddedilmeli");

    // Dosya duzeltilmeden arac duzelmez; kullanici dosyayi elle duzeltir.
    std::fs::write(
        depo.gorev_yolu(),
        "{\"surum\":1,\"guncelleme\":0,\"gorevler\":[]}",
    )
    .unwrap();

    // Dogru surumle devam: dongu kuran bagimlilik yine de reddedilir.
    calistir(&cli(&veri, ekle("A", 10, "normal", vec![]))).unwrap();
    calistir(&cli(&veri, ekle("B", 10, "normal", vec![]))).unwrap();
    calistir(&cli(
        &veri,
        Komut::Dep(BagimlilikKomut::Ekle(BagimlilikCiftArg {
            gorev: 1,
            bagimlilik: 2,
        })),
    ))
    .unwrap();
    let sonuc = calistir(&cli(
        &veri,
        Komut::Dep(BagimlilikKomut::Ekle(BagimlilikCiftArg {
            gorev: 2,
            bagimlilik: 1,
        })),
    ));
    assert!(sonuc.is_err(), "dongu reddedilmeli");
    let cikti = calistir(&cli(
        &veri,
        Komut::Dep(BagimlilikKomut::Agac(AgacArg { gorev: 1 })),
    ))
    .unwrap();
    assert!(cikti.contains("-> #2 B"), "{cikti}");
}

#[test]
fn bos_veri_klasorunde_ilk_kullanim_calisir() {
    let gecici = GeciciDizin::yeni("ilk-kullanim");
    let veri = gecici.yol().join("yeni-veri");
    assert!(!veri.exists());

    // Veri klasoru komutla birlikte olusur.
    let cikti = calistir(&cli(
        &veri,
        Komut::List(ListeArg {
            durum: None,
            etiket: None,
            ozet: true,
        }),
    ))
    .unwrap();
    assert_eq!(cikti, "kayitli gorev yok");
    assert!(veri.join("gorevler.json").exists() || veri.is_dir());

    // Bos depoda plan uretilebilir.
    let cikti = calistir(&cli(
        &veri,
        Komut::Plan(PlanArg {
            kapasite: 120,
            sabit: 0,
            tarih: None,
        }),
    ))
    .unwrap();
    assert!(cikti.contains("BUGUN SIGANLAR: 0 gorev"), "{cikti}");
}

#[test]
fn coklu_gorev_ekleme_tek_komutta() {
    let gecici = GeciciDizin::yeni("coklu-ekleme");
    let veri = gecici.alt("veri");
    let cikti = calistir(&cli(
        &veri,
        Komut::Add(EkleArg {
            basliklar: vec![
                "Birinci gorev".to_string(),
                "Ikinci gorev".to_string(),
                "Ucuncu gorev".to_string(),
            ],
            dakika: 40,
            oncelik: "normal".to_string(),
            etiket: vec!["hafta".to_string()],
            bagimlilik: Vec::new(),
        }),
    ))
    .unwrap();
    assert!(cikti.contains("#1"), "{cikti}");
    assert!(cikti.contains("#3"), "{cikti}");
    let depo = focuscompass::depo::Depo::yeni(&veri)
        .gorevleri_oku()
        .unwrap();
    assert_eq!(depo.uzunluk(), 3);
    assert_eq!(depo.etiket_sayisi("hafta").unwrap(), 3);
}
