# Kestirmeleri elle kurmak

*[English](SHORTCUTS.md)*

Normalde README'deki iCloud bağlantılarından kurmak yeterli. Bu sayfa kestirmeleri sıfırdan kurmak ya da nasıl çalıştıklarını görmek isteyenler için.

Üç kestirme var. Adları İngilizce, çünkü aynı kestirmeler her dildeki iPhone'da kullanılıyor. Apple'ın eylem adları (ör. "URL'nin İçeriğini Al") her telefonda o telefonun dilinde görünür.

| Kestirme | Ne yapar | Önerilen tetikleyici |
|---|---|---|
| **ClipBridge Setup** | PC'deki QR kodu okur, adresi ve token'ı iPhone'a kaydeder. Bir kez çalıştırılır. | — |
| **Send to PC** | iPhone panosunu (ya da paylaşılan öğeleri) PC'ye gönderir. | Arkaya Dokun ×2, Paylaş menüsü |
| **Get from PC** | PC panosunu iPhone panosuna alır. | Arkaya Dokun ×3 |

Ayarlar iCloud Drive'da `Kestirmeler/ClipBridge.txt` dosyasında durur. Kestirmelerin içinde adres ya da token yoktur, bu yüzden paylaşılabilirler.

Eylem adları Türkçe arayüze göre yazıldı, parantez içinde İngilizceleri var.

---

## 1. ClipBridge Setup

| # | Eylem | Ayar |
|---|---|---|
| 1 | **QR/Barkod Kodunu Tara** *(Scan QR/Barcode)* | — |
| 2 | **Dosyayı Kaydet** *(Save File)* | Girdi: **QR/Barkod Kodu** · **Nereye Kaydedileceğini Sor**: kapalı · **Alt Yol** *(Subpath)*: `ClipBridge.txt` · **Üzerine Yaz** *(Overwrite If File Exists)*: açık |
| 3 | **Bildirim Göster** *(Show Notification)* | `ClipBridge is ready` |

Dosya varsayılan olarak iCloud Drive → **Kestirmeler** *(Shortcuts)* klasörüne kaydedilir.

## 2. İki kestirmenin başına: ayarları okuma

"Send to PC" ve "Get from PC" şu eylemlerle başlar:

| # | Eylem | Ayar |
|---|---|---|
| 1 | **Dosya Al** *(Get File)* | Klasör: **Kestirmeler** · Yol: `ClipBridge.txt` · **Bulunamazsa Hata Ver**: açık |
| 2 | **Girdiden Sözlük Al** *(Get Dictionary from Input)* | Girdi: **Dosya** |
| 3 | **Sözlük Değerini Al** *(Get Dictionary Value)* | Anahtar: `url` |
| 4 | **Değişkeni Ayarla** *(Set Variable)* | Ad: `Address` |
| 5 | **Sözlük Değerini Al** | Anahtar: `token`. Girdiye dokunup 2. adımdaki **Sözlük**'ü seç; kendiliğinden yanlış dolar. |
| 6 | **Değişkeni Ayarla** | Ad: `Token` |

Dosya bulunamazsa kestirme hata verir. Bu, "ClipBridge Setup" kestirmesinin henüz çalıştırılmadığı anlamına gelir.

## 3. Send to PC

| # | Eylem | Ayar |
|---|---|---|
| 0 | **ⓘ** → **Paylaşma Sayfasında Göster** | Açık. Girdi türleri: Görüntüler, Dosyalar, Metin, Zengin Metin, URL'ler, Medya |
| 0 | "Girdi yoksa" | **Panoyu Al** *(Get Clipboard)* |
| 1–6 | Ayarları okuma (yukarıda) | |
| 7 | **Her Biriyle Yinele** *(Repeat with Each)* | Öğe: **Kestirme Girdisi** *(Shortcut Input)* |
| 8 | Yinelemenin içinde: **URL'nin İçeriğini Al** *(Get Contents of URL)* | URL: `Address` değişkeni + `/clip` · Yöntem: **POST** · Başlık: `X-Token` = `Token` değişkeni · İstek Gövdesi: **Form** → Tür **Dosya**, Anahtar `file`, Değer **Yinelenen Öğe** |
| 9 | Yinelemenin dışında: **Bildirim Göster** (isteğe bağlı) | `Sent to PC` |

```
Dosya Al  Kestirmeler/ClipBridge.txt
Girdiden Sözlük Al
Sözlük Değerini Al "url"    →  Değişkeni Ayarla Address
Sözlük Değerini Al "token"  →  Değişkeni Ayarla Token
Kestirme Girdisi'ndeki her öğe ile yinele
    URL'nin İçeriğini Al  [Address]/clip
        Yöntem: POST · Başlıklar: X-Token = [Token]
        İstek Gövdesi: Form → file (Dosya) = Yinelenen Öğe
Yinelemeyi Bitir
```

Form'da dikkat: `file` **anahtar** alanına, **Yinelenen Öğe** ise **değer** alanına girer. İkisi yer değiştirirse PC "Formda dosya alanı yok" der.

## 4. Get from PC

| # | Eylem | Ayar |
|---|---|---|
| 1–6 | Ayarları okuma (yukarıda) | |
| 7 | **URL'nin İçeriğini Al** | URL: `[Address]/clip` · Yöntem: **GET** · Başlık: `X-Token` = `[Token]` |
| 8 | **Panoya Kopyala** *(Copy to Clipboard)* | Girdi: **URL'nin İçeriği** |
| 9 | **Bildirim Göster** (isteğe bağlı) | `Copied from PC` |

## 5. Tetikleyiciler

Ayarlar → Erişilebilirlik → Dokunma → **Arkaya Dokun**:
- **Çift Dokunma** → Send to PC
- **Üçlü Dokunma** → Get from PC

iPhone 15 Pro ve sonrasında **Eylem Düğmesi** de kullanılabilir.

## 6. Kestirmeleri paylaşmak

Kestirmeler uygulamasında kestirmeye uzun bas → **Paylaş** → **iCloud Bağlantısını Kopyala**. Kestirmelerde gizli bilgi olmadığı için bağlantı herkese verilebilir.
