<p align="center"><img src="assets/tray.svg" width="72" alt=""></p>

<h1 align="center">ClipBridge</h1>

<p align="center">iPhone'da kopyala, Windows'ta yapıştır. Tersi de.<br>
<sub>Copy on your iPhone, paste on Windows, and back. No app on the phone, no cloud, no account.</sub></p>

---

ClipBridge, Windows'ta tepside çalışan küçük bir programdır (tek exe, ~4 MB). iPhone tarafında App Store uygulaması yok; iOS'un kendi **Kestirmeler** uygulaması kullanılır. Telefonun arkasına iki kez dokununca iPhone panosu PC'ye gider, üç kez dokununca PC panosu iPhone'a gelir. Her şey ev ağında kalır, hiçbir sunucudan geçmez.

## Özellikler

- **Metin, resim, dosya**: iki yönde de. Birden çok dosya PC'den iPhone'a ZIP olarak gelir.
- **iPhone fotoğrafları (HEIC)** Windows'ta yapıştırılabilir resme çevrilir. Fotoğraflar JPG, ekran görüntüleri kayıpsız PNG olarak kaydedilir.
- **Biçimli metin**: Notlar, Safari ya da Mail'den kopyalanan metin Word'e biçimiyle yapıştırılır.
- **Linkler** PC'de varsayılan tarayıcıda açılır (tepsi menüsünden kapatılabilir).
- **Sabit ağ adı**: PC ağda `clipbridge-<bilgisayar-adı>.local` adıyla duyurulur. Modem IP adresini değiştirse de kestirmeler çalışmaya devam eder.
- **QR ile kurulum**: Adres ya da token yazmak gerekmez.
- **Güncelleme bildirimi**: Yeni sürüm çıkınca tepsi menüsünde görünür.
- Boştayken işlemci kullanmaz. Windows panosunu izlemez; yalnızca istek gelince okur.

## Kurulum

### 1. Windows

1. [Releases](https://github.com/BatuhanCakmakk/clipbridge/releases/latest) sayfasından `ClipBridge.exe` dosyasını indir ve kalıcı bir klasöre koy (ör. `Belgeler\ClipBridge`). Ayarlar exe'nin yanındaki `ClipBridge-veri` klasöründe tutulur.
2. Çalıştır. SmartScreen uyarı verirse **Ek bilgi → Yine de çalıştır**. (Exe imzalı değil.)
3. Güvenlik duvarı sorarsa **Özel ağlar** için **İzin ver**.
4. İlk açılışta kurulum sayfası tarayıcıda açılır. Sonradan açmak için: tepsi simgesine sağ tıkla → **Kurulum / bağlantı bilgileri (QR)**.
5. İsteğe bağlı: tepsi menüsü → **Windows ile başlat**.

### 2. iPhone

1. Üç kestirmeyi ekle:
   - [ClipBridge Kurulum](ICLOUD_KURULUM)
   - [PC'ye Gönder](ICLOUD_GONDER)
   - [PC'den Al](ICLOUD_AL)
2. **ClipBridge Kurulum** kestirmesini çalıştır ve kamerayı PC ekranındaki QR koda tut.
3. Ayarlar → Erişilebilirlik → Dokunma → **Arkaya Dokun**:
   - **Çift Dokunma** → PC'ye Gönder
   - **Üçlü Dokunma** → PC'den Al
4. İlk gönderimde iOS yerel ağ izni ister: **İzin Ver**.

Kestirmeleri kendin kurmak istersen: [docs/KESTIRMELER.md](docs/KESTIRMELER.md)

## Kullanım

| Yapmak istediğin | Nasıl |
|---|---|
| iPhone → PC | iPhone'da kopyala, telefonun arkasına **iki kez** dokun, PC'de **Ctrl+V** |
| Fotoğraf / dosya göndermek | Fotoğraflar ya da Dosyalar'da **Paylaş → PC'ye Gönder** |
| PC → iPhone | PC'de kopyala, telefonun arkasına **üç kez** dokun, iPhone'da yapıştır |

iPhone'dan gelen dosyalar `İndirilenler\ClipBridge` klasörüne kaydedilir. Bu klasör kendiliğinden temizlenmez.

## Sorun giderme

| Belirti | Çözüm |
|---|---|
| Kestirme "sunucuya bağlanılamadı" diyor | iPhone ve PC aynı ağda mı? Windows'ta ağ profili **Özel** olmalı (Ayarlar → Ağ ve İnternet → ağın → Ağ profili türü). Güvenlik duvarında ClipBridge'e Özel ağ izni verilmiş olmalı. |
| `.local` adı çalışmıyor (bazı modemler mDNS'i engeller) | Kurulum sayfasındaki IP adresini kullan. `ClipBridge.txt` dosyasındaki `adres` değerini `http://192.168.x.x:8787` yap. |
| "Token hatalı" | QR kodu yeniden okut. Token `ClipBridge-veri\config.json` dosyasında. |
| "Dosya bulunamadı" (kestirme başında) | **ClipBridge Kurulum** henüz çalıştırılmamış. |
| Ne olduğunu görmek | Tepsi menüsü → **Günlüğü aç** |

## Güvenlik

- ClipBridge yalnızca ev ağından gelen istekleri kabul eder ve her istekte 20 karakterlik bir token ister. İnternete kapı açmaz; modemde port yönlendirme yapma.
- Bağlantı düz HTTP'dir. Ağdaki trafik şifrelenmez. Güvenmediğin ağlarda (kafe, otel, okul) kullanma.
- Kurulum sayfası yalnızca PC'nin kendisinden açılabilir. QR kodda token olduğu için ekran görüntüsünü paylaşma.
- Token'ı yenilemek için `config.json` dosyasındaki `token` değerini sil, ClipBridge'i yeniden başlat ve QR kodu tekrar okut.

## Ayarlar (`ClipBridge-veri\config.json`)

| Alan | Varsayılan | Açıklama |
|---|---|---|
| `port` | `8787` | Dinlenen port |
| `token` | rastgele | Kestirmelerin gönderdiği `X-Token` |
| `hostname` | `clipbridge-<pc-adı>` | Ağda duyurulan ad (`.local` eklenir) |
| `notifications` | `true` | Gelen içerik için Windows bildirimi |
| `open_links` | `true` | Tek bir link gelince tarayıcıda aç |
| `update_check` | `true` | Günde bir kez GitHub'da yeni sürüm denetle |

Değişiklikten sonra ClipBridge'i yeniden başlat.

## API

Kestirme yerine kendi aracını yazmak istersen:

| İstek | Açıklama |
|---|---|
| `GET /ping` | `ok` döner. Adres ve token'ı denemek için. |
| `POST /clip` | Gövde PC panosuna gider. Ham gövde ya da `multipart/form-data` (alan adı `file`). Dosya adı için `X-Filename` başlığı (URL kodlu) kullanılabilir. |
| `GET /clip` | PC panosu: metin, PNG, tek dosya ya da birden çok dosya için ZIP. Pano boşsa `204`. |

Hepsi `X-Token` başlığı ister.

```bash
curl -H "X-Token: <token>" --data-binary "merhaba" http://clipbridge-pc.local:8787/clip
```

## Derleme

Rust (stable, MSVC) gerekir:

```bash
cargo build --release
```

Çıktı: `target/release/clipbridge.exe`. Bir `v*` etiketi gönderildiğinde GitHub Actions exe'yi derleyip Releases'a koyar.

## Lisans

[MIT](LICENSE). Tepsi simgesi [Iconoir](https://iconoir.com) "send-diagonal" (MIT).
