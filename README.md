# FocusCompass (PusulaVakti)

> Pomodoro oturumu, görev bağımlılıkları, günlük kapasite hesabı ve kesinti günlüğüyle
> birlikte, **"bugün neyi yapamayacağını"** söyleyen çevrimdışı odak planlayıcı.

FocusCompass bir pomodoro sayacı değildir. Sayaç size kaç dakika geçtiğini söyler;
FocusCompass ise o dakikaların gününüzün geri kalanında **neye yetmeyeceğini** hesaplar ve
o işleri günün başında listeden ayırır. Bu davranış ürünün geri kalanını belirler:
günlük kapasite hesabı, görevler arası bağımlılık, kesinti günlüğü ve haftalık gerçekleşme raporu.

Tüm veri yerel dosyalarda tutulur. Araç hiçbir koşulda ağ bağlantısı açmaz, hesap istemez,
telemetri göndermez. Tek yürütülebilir dosyadır; kurulum adımı ve çalışma zamanı ön koşulu yoktur.

**Bu araç neden TypeFast değil** (kapsam ayrımı, KARAR D-011): FocusCompass yalnızca zaman
planlaması ve odak oturumu içindir. Metin şablonlama, kısayol çözümleme, pano geçmişi gibi
"genel metin" özellikleri kapsam dışıdır; bunlar 25 TypeFast'in alanıdır.

## Özellikler

- **Görev deposu ve açık biçim** — her görev; kimlik, başlık, tahmini süre (dakika), öncelik
  (`dusuk` / `normal` / `yuksek`), durum, etiketler ve bağımlılıklarla `gorevler.json`
  dosyasında, **sürümlü** şemayla (`"surum": 1`) tutulur. Dosya elle düzenlenebilir.
- **Görev bağımlılığı ve döngü tespiti** — bir görev başka görevlere bağlanabilir. Bağımlılık
  grafiği kendi içinde kurulur ve **DFS (beyaz/gri/siyah renkli)** ile denetlenir. Döngü
  oluşturan bir bağımlılık **kaydedilmez**, hata mesajı döngü yolunu gösterir. Bağımlılığı
  tamamlanmamış görev başlatılamaz; ertelenmiş bağımlılık da engeldir.
- **Pomodoro oturumu ve geçmiş** — yapılandırılmış odak/kısa mola/uzun mola süreleri, tam
  oturum, yarım kalan oturum (`birakildi`) ve kesinti günlüğü. Aynı anda yalnızca **bir**
  oturum açık olabilir; üst üste pomodoro reddedilir.
- **Günlük kapasite planlama** — kullanıcı günlük derin çalışma kapasitesini ve toplantı gibi
  sabit süreleri girer. Planlayıcı sabit süreleri düşer, uygun görevleri seçer ve **sığmayan
  her görevi gerekçesiyle ayrı satırda** listeler: `kapasite`, `bagimlilik`, `calisiyor`,
  `ertelendi`, `tahmin-yok`. Sığmayan işler sessizce düşürülmez.
- **Kesinti günlüğü** — oturum sırasında kesinti kaynak kısa etiketiyle (ör. `toplanti`)
  işaretlenir. Oturumun **gerçek odak süresi** (geçen süre − kesinti) ve toplam kesinti
  süresi ayrı raporlanır. Kesintilerin toplamı oturum süresini aşamaz.
- **Haftalık rapor** — ISO haftası bazında tamamlanan/ertelenen görev sayıları, toplam odak
  ve kesinti süresi, gün gün dağılım ve **tahmin ile gerçekleşenin yan yana** gösterimi.
  Aynı girdiyle **aynı sayılar** yeniden üretilir. `hafta-YYYY-Www.json` ve
  `hafta-YYYY-Www.md` olarak yazılır.
- **Günlük Markdown arşivi** — oturum kapandığında `gunluk/YYYY-MM-DD.md` dosyasına satır
  eklenir. Gün geçmişi arşivlenebilir ve elle okunabilir.
- **Atomik yazma** — görev dosyası ve raporlar geçici dosya + `rename` ile yazılır. Güç
  kesintisi yaşansa bile ana dosya ya eski içeriğini korur ya da yenisini bütün gösterir.
- **Çevrimdışı ve hesapsız** — ağ yüzeyi yoktur, kimlik doğrulama yoktur, güncelleme denetimi
  yoktur. Kullanım geçmişi yalnız yerel dosyalardadır.

## Kurulum

Gereksinim: Rust **1.74** veya üzeri (MSRV). Geliştirme ortamında `cargo 1.98.1` /
`rustc 1.98.1` ile derlenmiştir.

```powershell
cargo build --release
```

Oluşan tek dosya: `target\release\focuscompass.exe` (Unix'te `target/release/focuscompass`).
Bağımlılığı yoktur; başka bir klasörden çalıştırılabilir.

```powershell
cargo install --path .
```

Gerçek çıktı:

```
  Installing %USERPROFILE%\.cargo\bin\focuscompass.exe
   Installed package `focuscompass v0.1.0 (%USERPROFILE%\Desktop\Projeler\projects\23-focuscompass)` (executable `focuscompass.exe`)
```

```powershell
focuscompass --version
```

```
focuscompass 0.1.0
```

```powershell
focuscompass --veri D:\veri\focuscompass --tz-offset 180 list
```

## Kullanım

Aşağıdaki çıktıların tamamı Windows 11 + PowerShell 5.1 üzerinde `focuscompass 0.1.0`
ile **gerçekten çalıştırılmıştır**. Veri klasörü örneklerde kısaltılmış olarak
`$env:TEMP\fc-demo` yazılmıştır; kendi makinenizde `--veri` bayrağını değiştirin veya
bayrağı tamamen kaldırın (varsayılan: çalışma dizinindeki `veri/`).

### 1. Görev ekleme

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 add "Müşteri onayı" -m 30 --oncelik yuksek --tag "Mürekkep" --tag Onay
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 add "Yayın öncesi düzeltme" -m 45 --oncelik yuksek -d 1
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 add "Rapor taslağı" -m 90 --oncelik normal --tag rapor
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 add "Geçen yıl arşivi" -m 600 --oncelik dusuk
```

```
eklendi #1  Müşteri onayı  30 dk  oncelik=yuksek  durum=bekliyor  etiket=mürekkep,onay
eklendi #2  Yayın öncesi düzeltme  45 dk  oncelik=yuksek  durum=bekliyor  bagimlilik=#1
eklendi #3  Rapor taslağı  1 sa 30 dk  oncelik=normal  durum=bekliyor  etiket=rapor
eklendi #4  Geçen yıl arşivi  10 sa  oncelik=dusuk  durum=bekliyor
```

`#2`, `#1` bitmeden başlatılamaz (`-d 1`). Etiketler normalize edilir: büyük/küçük harf,
baştaki `#`, tekrarlar ve boşluklar temizlenir, sonuç alfabetik sıralanır.

### 2. Listeleme

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 list
```

```
#1    bekliyor    yuksek    30 dk  Müşteri onayı  etiket=mürekkep,onay
#2    bekliyor    yuksek    45 dk  Yayın öncesi düzeltme  bagimlilik=#1
#3    bekliyor    normal    90 dk  Rapor taslağı  etiket=rapor
#4    bekliyor    dusuk    600 dk  Geçen yıl arşivi
4 gorev
```

Filtreler (gerçek çıktılar):

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 list --durum bekliyor
```

```
#3    bekliyor    normal    90 dk  Rapor taslağı  etiket=rapor
#4    bekliyor    dusuk    600 dk  Geçen yıl arşivi
2 gorev
```

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 list --ozet
```

```
#1    Müşteri onayı
#2    Yayın öncesi düzeltme
#3    Rapor taslağı
#4    Geçen yıl arşivi
4 gorev
```

Girdi doğrulaması (tümü gerçekten çalıştırıldı):

```powershell
focuscompass --veri $env:TEMP\fc-demo add "Hatali" --dakika=-5
```

```
hata: sure negatif olamaz: -5 dk
```

```powershell
focuscompass --veri $env:TEMP\fc-demo add "Hatali" --oncelik cok
```

```
hata: gecersiz oncelik: 'cok' (dusuk | normal | yuksek)
```

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 start 99
```

```
hata: gorev bulunamadi: #99
```

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 plan --kapasite=-1 --tarih 2026-09-29
```

```
hata: kapasite veya sabit sure negatif olamaz: -1
```

### 3. Günlük kapasite planı

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 plan -c 120 -b 45 --tarih 2026-09-29
```

```
2026-09-29  kapasite 120 dk  sabit 45 dk  kullanilabilir 75 dk

BUGUN SIGANLAR: 1 gorev, 30 dk
  #1    yuksek    30 dk  Müşteri onayı

BUGUN SIGMAYANLAR: 3 gorev
  #2      45 dk  Yayın öncesi düzeltme  [bagimlilik]
        tamamlanmamis bagimlilik: #1 (bekliyor)
  #3      90 dk  Rapor taslağı  [kapasite]
        kalan 45 dk yetmiyor (gerekli 1 sa 30 dk)
  #4     600 dk  Geçen yıl arşivi  [kapasite]
        kalan 45 dk yetmiyor (gerekli 10 sa)

kalan 45 dk

uyari: kapasite asimi: 12 sa planlanabilir is, 1 sa 15 dk kullanilabilir kapasite
```

Bağımlılığı biten görev plana girer. `#1` tamamlandıktan sonra aynı komut şunu verir:

```
BUGUN SIGANLAR: 1 gorev, 45 dk
  #2    yuksek    45 dk  Yayın öncesi düzeltme

BUGUN SIGMAYANLAR: 2 gorev
  #3      90 dk  Rapor taslağı  [kapasite]
        kalan 30 dk yetmiyor (gerekli 1 sa 30 dk)
  #4     600 dk  Geçen yıl arşivi  [kapasite]
        kalan 30 dk yetmiyor (gerekli 10 sa)
```

### 4. Pomodoro oturumu, kesinti ve kapanış

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 start 2
```

```
hata: gorev baslatilamaz: #2 (bagimlilik #1 durumu: bekliyor)
```

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 start 1 -m 25
```

```
oturum #1 basladi: odak  planlanan 25 dk  gorev=#1 Müşteri onayı  gunluk=%USERPROFILE%\AppData\Local\Temp\fc-demo\oturumlar.jsonl
```

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 start 3 -m 25
```

```
hata: acik pomodoro oturumu var: #1; once onu durdurun
```

Kesinti, oturumun hangi dakikasında olduğunu bilerek kaydedilir (`--simdi` UTC epoch
saniyedir; verilmezse sistem saati kullanılır):

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 interrupt toplanti -m 3 --simdi 1790649004
```

```
kesinti kaydedildi: oturum #1  kaynak=toplanti  3 dk  toplam=3 dk (gecen 12 dk)
```

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 stop --bitis 1790649784
```

```
oturum #1 kapandi  #1 odak #1 [tamamlandi] 22 dk, 1 kesinti / 3 dk (kapali)  kesinti=3 dk  gorev=#1 durum=tamamlandi
```

25 dakikalık oturumdan 3 dakika kesildiği için **gerçek odak süresi 22 dakikadır**. Yarım
kalan oturum `birakildi` olarak raporlanır; görev tamamlanmış sayılmaz. Görevi bitirmek
yerine ertelemek için: `stop --ertele`.

### 5. Tam pomodoro döngüsü

`cycle` odak + mola döngüsünü baştan sona çalıştırır ve arada gerçekten bekler
(kayıt üretmeden denemek için `--kuru`):

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 cycle 2 --odak-dk 1 --mola-dk 1 -n 2 --mola-tipi uzun
```

```
dongu 1/2 odak: #2 odak #2 [tamamlandi] 1 dk (kapali)  1 dk
dongu 1/2 mola: uzun-mola  1 dk
dongu 2/2 odak: #4 odak #2 [tamamlandi] 1 dk (kapali)  1 dk
dongu 2/2 mola: uzun-mola  1 dk
toplam: 2 odak oturumu, 2 dk odak suresi  gorev=#2 durum=calisiyor
```

(Bu komut yukarıdaki örnekte gerçekten **240 saniye** çalıştı: 2 × (1 dk odak + 1 dk mola).)
Her odak oturumuna otomatik kesinti eklemek için `--kesinti-dk 4 --kesinti-kaynak toplanti`.
Tek döngüde (`-n 1`) görev `tamamlandi` olur; birden fazla döngüde `calisiyor` durumunda
kalır ve `stop` ile kapatılır.

### 6. Bağımlılık ağacı

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 dep agac 2
```

```
#2 Yayın öncesi düzeltme
  -> #1 Müşteri onayı [tamamlandi]
```

Döngü denemesi reddedilir ve **kaydedilmez**:

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 dep ekle 1 2
```

```
hata: bagimlilik dongusu: #1 -> #2 -> #1
```

### 7. Haftalık rapor

```powershell
focuscompass --veri $env:TEMP\fc-demo --tz-offset 180 report --hafta 2026-W40 --format ikisi
```

```
2026-W40 (2026-09-28 .. 2026-10-04): 1 oturum, 22 dk odak, 3 dk kesinti, 1 tamamlanan, 0 ertelenen
gerceklesme: %73  (tahmin 30 dk - gerceklesen 22 dk)
yazildi: %USERPROFILE%\AppData\Local\Temp\fc-demo\raporlar\hafta-2026-W40.json
yazildi: %USERPROFILE%\AppData\Local\Temp\fc-demo\raporlar\hafta-2026-W40.md
```

Markdown raporun içeriği (dosyadan okundu, kısaltılmadan):

```markdown
# FocusCompass haftalik rapor - 2026-W40

Donem: 2026-09-28 .. 2026-10-04
- Toplam oturum: 1
- Toplam odak suresi: 22 dk
- Toplam kesinti suresi: 3 dk
- Tamamlanan gorev: 1
- Ertelenen gorev: 0
- Tahmin (tamamlananlar): 30 dk
- Gerceklesme: %73

## Gunluk ozet

| Tarih | Gun | Oturum | Odak | Kesinti | Tamamlanan | Ertelenen | Tahmin |
|---|---|---|---|---|---|---|---|
| 2026-09-28 | Pazartesi | 0 | 0 | 0 | 0 | 0 | 0 |
| 2026-09-29 | Sali | 1 | 22 | 3 | 1 | 0 | 30 |
| 2026-09-30 | Carsamba | 0 | 0 | 0 | 0 | 0 | 0 |
| 2026-10-01 | Persembe | 0 | 0 | 0 | 0 | 0 | 0 |
| 2026-10-02 | Cuma | 0 | 0 | 0 | 0 | 0 | 0 |
| 2026-10-03 | Cumartesi | 0 | 0 | 0 | 0 | 0 | 0 |
| 2026-10-04 | Pazar | 0 | 0 | 0 | 0 | 0 | 0 |

## Kesinti kaynaklari

| Kaynak | Sure |
|---|---|
| toplanti | 3 dk |

> Bu rapor yerel dosyalardan yeniden uretilir; ayni girdiyle ayni sayilari verir.
```

### 8. Veri dosyaları

`gorevler.json` (atomik yazılır, sürümlü):

```json
{
  "surum": 1,
  "guncelleme": 1790648528,
  "gorevler": [
    {
      "id": 1,
      "baslik": "Müşteri onayı",
      "tahmini_dakika": 30,
      "oncelik": "yuksek",
      "durum": "tamamlandi",
      "etiketler": ["mürekkep", "onay"],
      "bagimliliklar": [],
      "olusturma": 1790648284,
      "guncelleme": 1790648287
    }
  ]
}
```

`oturumlar.jsonl` (her oturum bir satır; **eklenebilir**, asla tamamen yeniden yazılmaz):

```
{"sira":1,"gorev_id":1,"tur":"odak","durum":"tamamlandi","baslangic":1790648284,"bitis":null,"planli_dakika":25,"kesintiler":[]}
{"sira":1,"gorev_id":1,"tur":"odak","durum":"tamamlandi","baslangic":1790648284,"bitis":null,"planli_dakika":25,"kesintiler":[{"kaynak":"toplanti","dakika":3}]}
{"sira":1,"gorev_id":1,"tur":"odak","durum":"tamamlandi","baslangic":1790648284,"bitis":1790649784,"planli_dakika":25,"kesintiler":[{"kaynak":"toplanti","dakika":3}]}
{"sira":2,"gorev_id":2,"tur":"odak","durum":"tamamlandi","baslangic":1790648288,"bitis":1790648348,"planli_dakika":1,"kesintiler":[]}
{"sira":3,"gorev_id":null,"tur":"uzun-mola","durum":"tamamlandi","baslangic":1790648348,"bitis":1790648408,"planli_dakika":1,"kesintiler":[]}
{"sira":4,"gorev_id":2,"tur":"odak","durum":"tamamlandi","baslangic":1790648408,"bitis":1790648468,"planli_dakika":1,"kesintiler":[]}
{"sira":5,"gorev_id":null,"tur":"uzun-mola","durum":"tamamlandi","baslangic":1790648468,"bitis":1790648528,"planli_dakika":1,"kesintiler":[]}
```

Aynı `sira` birden fazla kez geçiyorsa **son kayıt geçerlidir**; bu sayede JSONL dosyası
eklenebilir kalır ve bir oturumun açılma/kesinti/kapanma anları bozulmadan biriktirilir.

`gunluk/2026-09-29.md`:

```markdown
# FocusCompass gunlugu - 2026-09-29

- 2026-09-29T02:18:04Z odak  #1 odak #1 [tamamlandi] 22 dk, 1 kesinti / 3 dk (kapali)
- 2026-09-29T02:20:08Z odak  #4 odak #2 [tamamlandi] 1 dk (kapali)
```

### 9. Tüm komutlar

```
focuscompass --help
```

```
FocusCompass (PusulaVakti): pomodoro oturumu, gorev bagimliligi cozumu ve gunluk kapasite planlamasini birlikte sunan terminal araci. Tum veri yerel dosyalarda tutulur; hicbir ag baglantisi acilmaz.

Usage: focuscompass.exe [OPTIONS] <COMMAND>

Commands:
  add        Yeni gorev ekler
  list       Gorevleri listeler
  start      Bir gorevde pomodoro oturumu baslatir
  stop       Acik oturumu kapatir ve gorevi tamamlar
  interrupt  Acik oturuma kesinti kaydeder
  dep        Bagimlilik yonetimi
  cycle      Bir veya birden fazla pomodoro dongusu calistirir
  plan       Gunluk kapasite planini uretir
  report     Haftalik raporu JSON ve Markdown olarak yazar
  help       Print this message or the help of the given subcommand(s)

Options:
      --veri <YOL>
          Veri klasoru. Varsayilan: calisma dizinindeki `veri/`

      --tz-offset <DAKIKA>
          Kullanici saat dilimi ofseti (dakika). Turkiye icin 180

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Test

```powershell
cargo test
```

Gerçek çıktı (Rust 1.98.1, Windows 11). Test adlarının tamamı listelenmiştir:

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.40s
     Running unittests src\lib.rs (target\debug\deps\focuscompass-830e8288e29a982e.exe)

running 123 tests
test cli::tests::acik_oturum_yoksa_stop_hata_verir ... ok
test cli::tests::add_list_akisi_calisir ... ok
test cli::tests::cycle_engelli_gorevi_baslatmaz ... ok
test cli::tests::cycle_kuru_kayit_uretir ... ok
test cli::tests::cycle_tek_dongu_gorevi_tamamlar ... ok
test cli::tests::dep_agac_bagimliliklari_gosterir ... ok
test cli::tests::gecersiz_oncelik_reddedilir ... ok
test cli::tests::dep_ekle_dongu_yaratmaz ... ok
test cli::tests::gecersiz_saat_dilimi_reddedilir ... ok
test cli::tests::interrupt_acik_oturuma_kayit_yapar ... ok
test cli::tests::interrupt_acik_oturum_yoksa_hata_verir ... ok
test cli::tests::negatif_sure_ile_ekleme_reddedilir ... ok
test cli::tests::list_durum_ve_etiket_filtresi_calisir ... ok
test cli::tests::plan_gecersiz_tarih_reddedilir ... ok
test cli::tests::plan_kapasite_ile_calisir ... ok
test cli::tests::report_gecersiz_format_reddedilir ... ok
test cli::tests::report_gecersiz_hafta_reddedilir ... ok
test cli::tests::report_iki_dosya_yazar ... ok
test cli::tests::start_bagimlilik_tamamlanmadiysa_reddedilir ... ok
test cli::tests::tur_secimi_donusumu_dogrudur ... ok
test cli::tests::start_stop_akisi_gunluk_dosyasi_yazar ... ok
test cli::tests::stop_erteleme_bayragi_durumu_degistirir ... ok
test cli::tests::ust_uste_oturum_reddedilir ... ok
test depo::tests::atomik_yazma_eksik_klasoru_olusturur ... ok
test depo::tests::atomik_yazma_yarim_dosya_birakmaz ... ok
test depo::tests::basarili_yazma_gecici_dosya_artigi_birakmaz ... ok
test depo::tests::bozuk_json_reddedilir ... ok
test depo::tests::dosya_yoksa_bos_depo_doner ... ok
test depo::tests::elle_bozulan_bagimlilik_denetlemede_yakalanir ... ok
test depo::tests::elle_yazilan_dongu_denetlemede_yakalanir ... ok
test depo::tests::gorevleri_yaz_ve_oku_gidis_donus_yapar ... ok
test depo::tests::gunluk_dosya_yolu_tarihle_eslesir ... ok
test depo::tests::gunluk_markdown_yazilir_ve_oku ... ok
test depo::tests::hazirla_alt_klasorleri_olusturur ... ok
test depo::tests::oturum_jsonl_bozuk_satir_hata_verir ... ok
test depo::tests::oturum_jsonl_satir_satir_eklenir ... ok
test depo::tests::saat_dilimi_bagimsiz_okuma ... ok
test depo::tests::surum_alani_eksikse_bozuk_json_sayilir ... ok
test depo::tests::surum_alani_kaydedilir ... ok
test depo::tests::surum_uyusmazligi_reddedilir ... ok
test gorev::tests::baslatilamayan_gorev_engellenen_bagimliligi_soyler ... ok
test gorev::tests::bos_baslik_reddedilir ... ok
test gorev::tests::bilinmeyen_bagimlilik_reddedilir ... ok
test gorev::tests::depo_denetleme_asiili_bagimliligi_yakalar ... ok
test gorev::tests::depo_durum_gecisi_uygular ... ok
test gorev::tests::durum_makinesi_gecerli_gecisleri_kabul_eder ... ok
test gorev::tests::durum_makinesi_gecersiz_gecisi_reddeder ... ok
test gorev::tests::durum_ve_etiket_filtresi_calisir ... ok
test gorev::tests::ekleme_sirasinda_hatali_bagimlilik_geri_alinir ... ok
test gorev::tests::ertelenmis_bagimlilik_gorevi_baslatir_engeller ... ok
test gorev::tests::etiket_normalizasyonu_kucult_sirala_tekrarla ... ok
test gorev::tests::gecersiz_etiket_reddedilir ... ok
test gorev::tests::iki_dugumlu_dongu_tespit_edilir ... ok
test gorev::tests::kendine_bagimli_gorev_reddedilir ... ok
test gorev::tests::negatif_tahmini_sure_reddedilir ... ok
test gorev::tests::sifir_tahmini_sure_kabul_edilir ... ok
test gorev::tests::siralama_oncelik_sure_kimlik ... ok
test gorev::tests::tamamlanan_bagimlilik_engel_olmaz ... ok
test gorev::tests::tekrar_bagimlilik_reddedilir ... ok
test gorev::tests::uc_dugumlu_dongu_tespit_edilir ... ok
test gorev::tests::zincir_bagimlilik_acar_ve_izgara_gosterir ... ok
test hata::tests::bos_kimlik_listesi_tire_olsun ... ok
test hata::tests::dongu_hatasi_okunur_yol_yazar ... ok
test hata::tests::dosya_hatasi_yolu_gosterir ... ok
test hata::tests::gecersiz_gecis_hatasi_iki_durumu_yazar ... ok
test kapasite::tests::bagimlilik_tamamlanmadan_gorev_plana_girmez ... ok
test kapasite::tests::bos_depo_bos_plan_uretir ... ok
test kapasite::tests::calisan_ve_ertelenen_gorevler_plana_girmez ... ok
test kapasite::tests::ertelenmis_bagimlilik_plani_engeller ... ok
test kapasite::tests::kapasiteye_sigan_gorevler_plana_girer ... ok
test kapasite::tests::kapasiteyi_asan_gorev_isaretlenir_ve_uyari_uretir ... ok
test kapasite::tests::negatif_kapasite_reddedilir ... ok
test kapasite::tests::oncelik_kapasiteyi_once_dagitir ... ok
test kapasite::tests::plan_metni_gerekceleri_gosterir ... ok
test kapasite::tests::sabit_sure_kapasiteden_dusulur ... ok
test kapasite::tests::sifir_kapasite_uyari_uretir ... ok
test kapasite::tests::tahmini_sure_sifir_olan_gorev_plana_girmez ... ok
test kapasite::tests::tamamlanan_gorev_plana_girmez ... ok
test oturum::tests::acik_oturum_bitirilemezse_hata_verir ... ok
test oturum::tests::acilis_kapanis_durumu_yalin ... ok
test oturum::tests::acilis_oturumu_toplamlara_girmez ... ok
test oturum::tests::ardisik_oturumlar_cakismaz ... ok
test oturum::tests::ayni_sira_iki_kez_varsa_son_kayit_gecerlidir ... ok
test oturum::tests::bitis_baslangictan_once_reddedilir ... ok
test oturum::tests::cakisik_kayit_ciftleri_tespit_edilir ... ok
test oturum::tests::gece_yari_asil_oturum_basladigi_gune_yazilir ... ok
test oturum::tests::kapanmis_oturum_tekrar_kapatilamaz ... ok
test oturum::tests::kaynak_dagilimi_hesaplanir ... ok
test oturum::tests::kesinti_gecen_sureyi_asan_reddedilir ... ok
test oturum::tests::kesinti_toplami_ve_odak_suresi_ayrilir ... ok
test oturum::tests::mola_oturumu_odak_toplamina_girmez ... ok
test oturum::tests::negatif_planli_sure_reddedilir ... ok
test oturum::tests::sifir_planli_sure_reddedilir ... ok
test oturum::tests::sifir_ve_negatif_kesinti_reddedilir ... ok
test oturum::tests::tur_ayrimasi_ve_adlari ... ok
test oturum::tests::ustelte_oturum_baslatilamaz ... ok
test oturum::tests::yarim_kalan_oturum_birakildi_isaretlenir ... ok
test rapor::tests::gerceklesme_yuzdesi_tahminle_hesaplanir ... ok
test rapor::tests::gun_adi_ve_tarih_uyumu ... ok
test rapor::tests::hafta_disi_oturumlar_raporu_etkilemez ... ok
test rapor::tests::hafta_yedi_gun_doner_ve_aralik_dogru ... ok
test rapor::tests::haftalik_rapor_toplamlari_dogru ... ok
test rapor::tests::json_rapor_yapisal_gecerli_ve_geri_yuklenebilir ... ok
test rapor::tests::kesinti_toplamlari_ve_kaynak_dagilimi ... ok
test rapor::tests::markdown_tahmin_ve_gercekleseni_yan_yana_gosterir ... ok
test rapor::tests::rapor_ayni_girdiyle_ayni_sayilari_uretir ... ok
test rapor::tests::tahmin_yokken_yuzde_sifirdir ... ok
test zaman::tests::bilinen_tarihler_gidis_donus_yapar ... ok
test zaman::tests::gecersiz_hafta_reddedilir ... ok
test zaman::tests::gecersiz_saat_dilimi_reddedilir ... ok
test zaman::tests::gecersiz_tarihler_elenir ... ok
test zaman::tests::gun_adi_pazartesi_baslar ... ok
test zaman::tests::gun_siniri_gece_yari_yeni_gundur ... ok
test zaman::tests::gun_siniri_gun_baslangic_ve_sonu_tutarli ... ok
test zaman::tests::hafta_ayir_gidis_donus_yapar ... ok
test zaman::tests::hafta_yedi_gun_doner ... ok
test zaman::tests::iso8601_utc_zaman_damgasini_yazar ... ok
test zaman::tests::iso_hafta_yil_dongusu_dogru ... ok
test zaman::tests::negatif_gun_no_cekirilir ... ok
test zaman::tests::referans_gun_sifirdir ... ok
test zaman::tests::saat_dilimi_ofseti_gun_degistirir ... ok
test zaman::tests::sure_yaz_kisa_ve_uzun_bicim ... ok
test zaman::tests::tarih_bicim_kurallari_uygulanir ... ok

test result: ok. 123 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.96s

     Running unittests src\main.rs (target\debug\deps\focuscompass-b2dcf5b4cad8f028.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests\entegrasyon.rs (target\debug\deps\entegrasyon-d2c4c07f2d6c1694.exe)

running 7 tests
test bos_veri_klasorunde_ilk_kullanim_calisir ... ok
test bozuk_dosya_araci_kilitlemez_ve_silinmez ... ok
test coklu_gorev_ekleme_tek_komutta ... ok
test gunun_tam_akisi_calisir ... ok
test haftalik_rapor_json_ve_markdown_dosyalari_uretir ... ok
test surum_uyusmazligi_ve_asiili_bagimlilik_yonetimi ... ok
test ust_uste_pomodoro_ve_kesinti_gunlugu_akisi ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s

   Doc-tests focuscompass

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**test sonucu: okunan 130; geçen 130; başarısız 0**

Kapsanan kenar durumları:

| Alan | Test edilen senaryolar |
|---|---|
| Bağımlılık döngüsü | 2 düğümlü halka, 3 düğümlü halka, kendine bağımlı görev, üç düğümlü halkanın yolunun (`#1 -> #2 -> #3 -> #1`) raporlanması, elle bozulan dosyadaki döngünün `denetle` ile yakalanması, **reddedilen bağımlılığın kaydedilmemesi** |
| Pomodoro | Üst üste oturum açılamaması, elle bozulmuş günlükte örtüşen oturum çiftlerinin bulunması, ardışık oturumların çakışmaması, yarım kalan oturumun `birakildi` sayılması, kapanmış oturumun tekrar kapatılamaması, bitişin başlangıçtan önce olamaması, mola oturumlarının odak toplamına girmemesi |
| Gün sınırı | Yerel gece yarısının yeni güne sayılması, `23:59:59` / `00:00:00` sınırı, `gun_baslangic`–`gun_sonu` tutarlılığı, saat dilimi ofsetinin gün değiştirmesi, gece yarısını aşan oturumun **başladığı** güne yazılması |
| Süre | Negatif tahmini süre, negatif oturum süresi, sıfır oturum süresi, sıfır tahminli görevin plana girmemesi, negatif kesinti, sıfır kesinti, geçen süreyi aşan kesinti |
| Sürüm / bozuk veri | `surum` uyuşmazlığı, `surum` alanının eksikliği, bozuk JSON, bozuk JSONL satırı (satır numarası hatada), elle yazılan asılı bağımlılık, **bozuk dosyanın silinmemesi ve düzeltilince aracın toparlanması** |
| Atomik yazma | Yazma başarısız olduğunda hedef dosyanın **hiç değişmemesi**, başarılı yazma sonrası geçici dosya artığı kalmaması, eksik klasörün oluşturulması |
| Etiket | Normalizasyon (küçük harf, `#` temizliği, boşluk → `-`, tekrar silme, alfabetik sıralama), Türkçe `I`/`İ` dönüşümü, geçersiz karakter reddi |
| Durum makinesi | 10 geçerli geçiş, `tamamlandi -> ertelendi` ve `ertelendi -> tamamlandi` reddi, gerçek depo üzerinde geçersiz geçiş reddi |
| Kapasite | Sığanların seçilmesi, kapasite aşanların işaretlenmesi, aşım uyarısının üretilmesi, sıfır kapasite uyarısı, sabit sürenin düşülmesi, önceliğin kapasiteyi önce dağıtması, bağımlılık/ertelenmiş/çalışıyor/tamamlandı ayrımları |
| Rapor | Yedi günlük aralık, hafta toplamları, tamamlanan/ertelenen sayımı, kesinti toplamı ve kaynak dağılımı, tahmin/gerçekleşen yüzdesi, JSON'un geri yüklenebilirliği, Markdown'da tahmin ile gerçekleşenin yan yana çıkması, raporun tekrarlanabilirliği, hafta dışı oturumların raporu etkilememesi |
| Entegrasyon | Tam gün akışı (ekle → bağımlılık → planla → cycle → planla), üst üste pomodoro + kesinti + kapanış, rapor dosyalarının içeriği, bozuk dosya kurtarma, sürüm uyuşmazlığı, boş klasörde ilk kullanım, çoklu görev ekleme |

Ek kalite kapıları:

```powershell
cargo clippy --all-targets -- -D warnings   # temiz
cargo fmt --check                          # temiz
cargo tree                                 # yalnizca serde, serde_json, clap ve gecislileri
```

## Proje Yapısı

```
23-focuscompass/
├── Cargo.toml              # bagimliliklar ve gerekce yorumlari
├── Cargo.lock              # uretilir, commit edilir
├── LICENSE.txt             # MIT, 2026
├── README.md
├── .gitignore
├── src/
│   ├── lib.rs              # cekirdek: modul yonlendirmesi, #![forbid(unsafe_code)]
│   ├── main.rs             # CLI kabugu: arguman ayristirma + hata yonlendirme
│   ├── hata.rs             # tek hata turu, elle Display + std::error::Error
│   ├── zaman.rs            # UTC epoch + saat dilimi, sivil tarih, ISO hafta
│   ├── gorev.rs            # gorev deposu, durum makinesi, bagimlilik grafigi + DFS
│   ├── oturum.rs           # pomodoro oturumu, oturum gunlugu, kesinti gunlugu
│   ├── kapasite.rs         # gunluk kapasite planlayicisi
│   ├── depo.rs             # surumlu JSON, JSONL, atomik yazma
│   ├── rapor.rs            # haftalik rapor (JSON + Markdown)
│   └── cli.rs              # clap alt komutlari ve komut yurutucusu
└── tests/
    ├── ortak/mod.rs        # gecici dizin yardimcisi (Drop ile temizlik)
    └── entegrasyon.rs      # uctan uca akis testleri
```

Katmanlar tek yönlü bağımlılık gösterir: `hata` hiçbir şeye bağlı değildir; `gorev`,
`oturum`, `kapasite` ve `rapor` yalnızca `hata` ve `zaman` üzerine kurulur; `depo` bu
tipleri diske yazar; `cli` hepsini birbirine bağlar.

## Yapılandırma

Yapılandırma dosyası yoktur; her ayar komut satırındadır. Bu, taşınabilir kullanımda
(çıkarılabilir disk, program dizini salt okunur) verinin nerede tutulacağını taşımayı
gereksiz kılar.

| Bayrak / değişken | Kapsam | Varsayılan | Etkisi |
|---|---|---|---|
| `--veri <YOL>` | tüm komutlar | çalışma dizinindeki `veri/` | Veri klasörünü belirler. Komut çalıştığında yoksa `veri/`, `veri/gunluk/` ve `veri/raporlar/` oluşturulur. İçine `gorevler.json`, `oturumlar.jsonl`, `gunluk/`, `raporlar/` yazılır. |
| `--tz-offset <DAKIKA>` | tüm komutlar | `180` (Türkiye, UTC+3) | Gün sınırlarını, tarih gösterimini ve `YYYY-MM-DD` çıktısını belirler. Kabul aralığı `-840..=840` (−14:00..+14:00). Yaz saati uygulaması modellenmez; ofset sabittir. |
| `add -m/--dakika <N>` | `add` | `25` | Görevin tahmini süresi (dakika). Negatif reddedilir; `0` "süresi bilinmiyor" anlamına gelir ve kapasite hesabına katılmaz. |
| `add --oncelik <SEVIYE>` | `add` | `normal` | `dusuk` \| `normal` \| `yuksek`. Kapasite dağıtımında öncelik sırayı belirler. |
| `add -t/--tag <ETIKET>` | `add` | yok | Tekrarlanabilir. Normalize edilir; geçersizse komut reddedilir. |
| `add -d/--depends <ID>` | `add` | yok | Tekrarlanabilir. Döngü oluşturursa komut reddedilir ve görev hiç eklenmez. |
| `list --durum <DURUM>` | `list` | yok | `bekliyor` \| `calisiyor` \| `tamamlandi` \| `ertelendi`. |
| `list --tag <ETIKET>` | `list` | yok | Etikete göre filtre. |
| `list --ozet` | `list` | yok | Yalnızca kimlik ve başlık gösterir. |
| `start -m/--dakika <N>` | `start` | `25` | Oturumun planlanan süresi (1..=240). |
| `start --tur <TUR>` | `start` | `odak` | `odak` \| `kisa` \| `uzun`. `odak` dışındaki türler görev durumunu değiştirmez. |
| `stop --ertele` | `stop` | yok | Görevi `tamamlandi` yerine `ertelendi` yapar. |
| `stop --bitis <EPOCH>` | `stop` | sistem saati | Oturumun biteceği an. Kayıt düzeltme ve test amaçlıdır. |
| `interrupt --simdi <EPOCH>` | `interrupt` | sistem saati | Kesintinin oluştuğu an. Geçen süre bu değerden hesaplanır. |
| `cycle --odak-dk <N>` | `cycle` | `25` | Odak süresi (1..=180). |
| `cycle --mola-dk <N>` | `cycle` | `5` | Mola süresi (0..=120). `0` molayı atlar. |
| `cycle -n <ADET>` | `cycle` | `1` | Döngü sayısı (1..=12). |
| `cycle --mola-tipi <TUR>` | `cycle` | `kisa` | `kisa` \| `uzun`. |
| `cycle --kesinti-dk <N>` | `cycle` | yok | Her odak oturumuna otomatik kesinti ekler. |
| `cycle --kuru` | `cycle` | yok | Beklemez; oturumları planlanan süreyle yazar. |
| `plan -c/--kapasite <N>` | `plan` | `180` | Günlük derin çalışma kapasitesi (dakika). |
| `plan -b/--sabit <N>` | `plan` | `0` | Toplantı/ulaşım gibi sabit süreler; kapasiteden düşülür. |
| `plan --tarih <YYYY-MM-DD>` | `plan` | bugün | Planlanacak gün. Geçersiz tarih reddedilir. |
| `report --hafta <YYYY-Www>` | `report` | bulunulan hafta | ISO haftası. `2026-40` gibi biçimsiz girdi reddedilir. |
| `report --format <BIÇIM>` | `report` | `ikisi` | `json` \| `md` \| `ikisi`. |
| `report --cikti <YOL>` | `report` | `--veri` | Rapor dosyalarının yazılacağı klasör. |
| `dep ekle/kaldir <gorev> <bagimlilik>` | `dep` | — | Bağımlılık ekler/kaldırır. |
| `dep agac <gorev>` | `dep` | — | Geçişli bağımlılık ağacını ve durumları gösterir. |

Veri dosyası şeması:

```
veri/
├── gorevler.json     surumlu ({ "surum": 1, "guncelleme": <epoch>, "gorevler": [...] })
├── oturumlar.jsonl   her satır bir oturum (JSON); eklenebilir
├── gunluk/           YYYY-MM-DD.md  (oturum kapanis satirlari)
└── raporlar/         hafta-YYYY-Www.json, hafta-YYYY-Www.md
```

## Bilinen Sınırlamalar

**Ölçülmemiş iddialar.** Fikir raporundaki bellek bütçesi (310 MB tepe RSS), açılış süresi
(900 ms) ve "gerçekçi plan" vaadi **hiç ölçülmedi**. Bu proje 5.000 görev / 2 yıllık kayıt
yükünde performans ölçümü yapmamıştır. Kapasite planlayıcısının ağırlıkları (öncelik →
tahmini süre → kimlik) kullanıcı girdisine bağlıdır ve doğruluğu ölçülmemiştir.

**MANIFEST kartındaki ertelenenler (uygulanmadı):**

- Yerel bildirim (işletim sistemi) — oturum bitişi yalnız terminal çıktısıdır, ses yoktur.
- Takvim dosyası okuma (`.ics`) — toplantı süreleri elle `-b/--sabit` ile girilir.
- Şifreli çalışma modu — veri düz metindir; kullanıcı korumayı işletim sistemine bırakır.
  **Düz metin bir görev başlığı işveren veya aile üyesi tarafından görülebilir.**
- Arayüz panelleri ve takvim görünümü — çıktı düz metindir (KARAR D-010: pencere katmanı
  tüm projeler için kalıcı olarak yasak).

**Teknik sınırlar:**

- **Yaz saati modellenmez.** `--tz-offset` sabit bir ofsettir; Türkiye'de yıl içinde iki kez
  saat değiştiğinde o günlerin UTC sınırı kayar. Oturum *süreleri* bundan etkilenmez
  (epoch farkı kullanılır), yalnız gün sınırı ve tarih gösterimi etkilenir.
- **Görev geçmişi tutulmaz.** Haftalık rapordaki "tamamlanan/ertelenen" sayıları, görevin
  `guncelleme` zaman damgasının rapor aralığına düştüğü günlere göre hesaplanır. Bir görev
  hafta içinde iki kez durum değiştirdiyse yalnız son durum sayılır.
- **Aynı anda tek oturum.** İki paralel pomodoro çalıştırılamaz; bu, kural gereği
  reddedilir. Çoklu görev üstünde çalışma yoktur.
- **Kesinti kaynağı serbest metin değildir** ama kısa etiket olarak serbestçe yazılabilir;
  hassas bilgi girilmemelidir.
- **`cycle` bekleme yapar.** Gerçek pomodoro çalıştırmak komutu bloke eder; `--kuru`
  ile kayıt üretmeden denenebilir.
- **Görev dosyası elle düzenlenebilir ama denetim sıkıdır:** eksik alan, bilinmeyen
  bağımlılık veya döngü varsa dosya **reddedilir** (satır işaretlenip atlanmaz). Dosya
  asla silinmez veya düzeltilmez; kullanıcı düzeltir.
- **Ölçek sınırları test edilmedi.** 5.000 görevlik bütçe raporun hedefidir; testler
  küçük veri kümeleriyle sınırlıdır. Bağımlılık denetimi özyinelemeli DFS'tir; çok derin
  zincirlerde yığın derinliği artar (5.000 düğüm güvenli aralıktadır, ölçülmedi).
- **JSONL okuyucu tek bozuk satırda tüm günlüğü reddeder** (satır numarası hatada verilir).
  Bu, sessiz veri kaybını önlemek içindir; kurtarma aracı yoktur.

## Gelecek Geliştirmeler

- Yerel oturum bitiş bildirimi (işletim sistemi ekran bildirimi; görev adı bildirimde yer almaz).
- `.ics` okuma (yalnız okuma; okunamazsa elle girilen sürelerle devam).
- Kişisel kapasite tahmini: geçmiş oturumlardan öğrenilen bir güven aralığı ile ilk haftalarda
  daha muhafazakâr planlama.
- Görev geçmişi (durum değişikliği olay günlüğü) — haftalık raporun "tamamlanan" sayısını
  bugünkü yaklaşımdan gerçek sayıma taşıyacak.
- Şifreli çalışma modu (raporun v2 hedefi); açık metin modu her zaman korunacaktır.
- Veri dışa aktarma: haftalık rapor için statik HTML/SVG sayfası (raporun b16 açık sorusu 4).
- Kapasite planlama kurallarının yapılandırılabilir hale getirilmesi (raporun R6 riski).

## Troubleshooting

**1) `hata: veri dosyasi surumu uyusmuyor: dosyada 7, bu surum 1 bekliyor`**

- Belirti: araç hiçbir komutu çalıştırmıyor, veri dosyasını okuyamıyor.
- Neden: `gorevler.json` elle düzenlendi ve `"surum"` alanı farklı bir şema sürümü gösteriyor.
- Çözüm: dosyayı bir metin düzenleyicide açın, `"surum"` değerini `1` yapın. Görev
  kayıtlarınızın geri kalanı bozulmadan geri yüklenir. Dosyayı silmeyin; araç dosyayı
  ne siler ne de değiştirir.

**2) `hata: bozuk JSON: ...` / `hata: bozuk JSONL satiri ...: satir 12 (...)`**

- Belirti: komut hiçbir çıktı vermeden hata ile duruyor.
- Neden: dosya yarım kalmış ya da elle bozulmuş. JSONL'de hata **satır numarasıyla** verilir.
- Çözüm: hata mesajındaki satır numarasını `oturumlar.jsonl` içinde bulun ve o satırı
  silin ya da düzeltin. Görev dosyası sürümlü ve atomik yazıldığı için güç kesintisi
  nedeniyle yarım kalmış olması beklenmez; bu durum elle düzenleme kaynaklıdır.

**3) `hata: bagimlilik dongusu: #2 -> #1 -> #2`**

- Belirti: `dep ekle` veya `add -d` komutu bağımlılığı kaydetmiyor.
- Neden: yeni bağımlılık grafiği kapalı bir döngü oluşturuyordu. Araç kilitlenmez,
  isteği reddeder ve veri değişmez.
- Çözüm: hata mesajındaki yolu okuyun ve zincirin hangi halkasını kırmak istediğinize
  karar verin (`dep kaldir`). Kırmak istediğiniz halkayı `dep agac <gorev>` ile görün.

**4) `hata: acik pomodoro oturumu var: #1; once onu durdurun`**

- Belirti: yeni bir oturum başlatılamıyor; `cycle` de aynı hatayı verir.
- Neden: JSONL günlüğünde kapanmamış bir oturum kaydı var. Bu, programın kapanması ya da
  `stop` komutunun çalıştırılmaması nedeniyle olabilir.
- Çözüm: `stop` çalıştırın. Oturum çok eskiyse ve gerçekten yarım kaldıysa
  `stop --bitis <epoch>` ile istediğiniz bitiş anını elle verin; ya da `oturumlar.jsonl`
  içindeki son satırı düzenleyin (`"bitis": <epoch>`).

**5) `hata: gorev baslatilamaz: #2 (bagimlilik #1 durumu: bekliyor)`**

- Belirti: `start`/`cycle` reddediliyor.
- Neden: bağımlı görevin henüz `tamamlandi` durumunda değil. `bekliyor`, `calisiyor` ve
  **ertelendi** durumlarındaki bağımlılıklar da engeldir; ertelenen bir iş bitmiş sayılmaz.
- Çözüm: `dep agac <gorev>` ile engelin hangi görev olduğunu görün, ardından `#1` görevini
  tamamlayın (`start`/`cycle` + `stop`) veya bağımlılığı kaldırın (`dep kaldir`).

**6) Rapor hep "0 tamamlanan" gösteriyor**

- Belirti: haftalık raporda tamamlanan görev sayısı sıfır.
- Neden: araç görev geçmişi tutmaz; sayı, görevin `guncelleme` zaman damgasının rapor
  aralığına düşmesinden hesaplanır. Görevi bu haftadan önce tamamladıysanız sayılmaz.
- Çözüm: `--hafta` değerinin doğru ISO haftası olduğundan emin olun (`hafta_yaz` çıktısını
  `list` çıktısıyla karşılaştırın). Aynı haftada tamamlanan görevler `guncelleme` damgası
  o haftaya düşüyorsa rapor doğru çalışır.

**7) `--tz-offset` vermediğimde tarih bir gün yanlış görünüyor**

- Belirti: `plan` çıktısındaki tarih ya da rapordaki gün dağılımı kaymış.
- Neden: varsayılan ofset `180` (UTC+3). Farklı saat dilimindeyseniz gün sınırı kayar.
- Çözüm: `--tz-offset` değerini dakika cinsinden verin (`--tz-offset 0` UTC,
  `--tz-offset -300` New York). Kabul aralığı `-840..=840`'dür.

## Atıflar

Bu proje aşağıdaki açık kaynak çalışmaları ve standartlara dayanır. Doğrudan kopyalanan
üçüncü taraf kodu yoktur.

- **Pomodoro yöntemi** — Francesco Cirillo, *The Pomodoro Technique*, <https://www.pomodoro-technique.com/>
  (varsayılan 25 dakika odak / 5 dakika mola düzeni bu kaynaktan alınmıştır)
- **RFC 3339 — Date and Time on the Internet: Timestamps** (ISO 8601 tabanlı zaman damgası
  biçimi, `date-time` üretimi) — <https://www.rfc-editor.org/rfc/rfc3339>
- **RFC 3339 §5.6 bölümü, "ISO 8601 Date and Time Representation"** — <https://www.rfc-editor.org/rfc/rfc3339#section-5.6>
- **ISO 8601 — Date and time representations** (hafta numarası ve tarih biçimi; ISO 8601'in
  ücretsiz açıklaması için) — <https://www.iso.org/iso-8601-date-and-time-format.html>
- **Howard Hinnant, "chrono-Compatible Low-Level Date Algorithms"** (kamu malı; `zaman.rs`
  içindeki sivil tarih ⇄ gün sayısı dönüşümü bu yöntemle uygulanmıştır) —
  <https://howardhinnant.github.io/date_algorithms.html>
- **Rust standart kütüphanesi — `std::time`** (`SystemTime`, `Duration`, `thread::sleep`) —
  <https://doc.rust-lang.org/std/time/>
- **serde** — <https://serde.rs/> ve <https://github.com/serde-rs/json> (`serde_json`)
- **clap** (komut satırı çözümleyici ve `derive` makroları) — <https://docs.rs/clap/>
- **Cargo yerel rehberi** — <https://doc.rust-lang.org/cargo/>
- **Rust 2021 sürüm rehberi** — <https://doc.rust-lang.org/edition-guide/edition-2021/>
- **Fikir raporunun kendisi** (iç tasarımın kaynağı, yerel dosya — URL değildir):
  `%USERPROFILE%\Desktop\Fikirler\23-pusula-vakti-odak-planlayici.html`
- **Proje teknik kartı** (kapsam ve sapma kararları, yerel dosya):
  `%USERPROFILE%\Desktop\Projeler\MANIFEST.md`, kart 23

## Lisans

MIT lisansı. Tam metin: [LICENSE.txt](LICENSE.txt).

Telemetri yok · çökme raporu yok · kimlik doğrulama yok · ağ bağlantısı yok.
