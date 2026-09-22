# MD Preview

> Apple platformu (macOS / iOS / iPadOS) indirmeleri, paket derlemeleri ve imzalama askıya alındı. Aşağıda anlatılan Apple özellikleri yalnızca korunan kaynak koda referanstır.

**[English](README.md) · [简体中文](README_zh.md) · Türkçe**

[![GitHub stars](https://img.shields.io/github/stars/vorojar/md-preview)](https://github.com/vorojar/md-preview/stargazers)
[![Release](https://img.shields.io/github/v/release/vorojar/md-preview)](https://github.com/vorojar/md-preview/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20Android-lightgrey)](https://github.com/vorojar/md-preview/releases)
[![Binary size](https://img.shields.io/badge/binary-~5MB-green)](https://github.com/vorojar/md-preview/releases)

> Birden çok Markdown dosyası, tek hafif pencere. Yerel belge bağlantılarını takip et, sayaçları incele, sayfayı yakınlaştır, otomatik kayıtla düzenle — koca bir IDE açmadan.

MD Preview; **Rust** ve masaüstünde sistemin kendi **WebView**'i üzerine kurulu, hızlı, yerel-öncelikli bir Markdown önizleyici ve hızlı düzenleyicidir; ayrıca Dosyalar, WeChat, WeCom ve sistem paylaşım sayfalarından Markdown açmak için native iOS ve Android kabukları da vardır. Chromium içermez, Electron gerektirmez, tüm render varlıklarını çevrimdışı tutar. Birden fazla yerel belgeyi sekme olarak açın, yeniden başlattıktan sonra aynı aktif belgeye dönün ya da macOS'ta Finder'dan bir Markdown dosyası oluşturup hemen yazmaya başlayın.

![MD Preview ekran görüntüsü](https://vorojar.github.io/md-preview/hero.jpg)

## Neden Var

Yapay zekâ kodlama araçları artık bolca Markdown üretiyor: `README.md`, `plan.md`, görev tanımları, mimari notları, değişiklik günlükleri, KaTeX formülleri, Mermaid diyagramları. Çoğu Markdown aracı hâlâ ya tam bir yazım stüdyosu ya da bir editör eklentisi. MD Preview bilinçli olarak daha küçük:

- **Hızlı açılır** — native binary, sistemin WebView'i, gömülü tarayıcı çalışma zamanı yok.
- **Yerelde kalır** — Markdown, syntax highlighting, matematik ve diyagramlar kendi makinenizde render edilir.
- **Belgeleri bir arada tutar** — birden fazla Markdown/metin dosyasını tek sekmeli pencerede açın, oturuma sonra devam edin.
- **Dokümantasyon klasörlerinde gezinir** — yerel Markdown/metin dosyalarına göreli veya mutlak bağlantılar önizlemeden çıkmadan sekme açar veya etkinleştirir.
- **Dolambaçsız düzenler** — sekme çubuğundan veya Finder'dan Markdown oluşturun, hemen yazın, gecikmeli otomatik kayıt değişikliği kalıcı hale getirsin.
- **Kendi hızınızda okutur** — önizleme ile kaynak arasında kaydırma ilerlemesini korur, canlı karakter sayaçları gösterir, yalnızca belge içeriğini yakınlaştırır.
- **Dış düzenlemeleri takip eder** — dosyayı Vim, VS Code, Cursor, Zed veya başka bir şeyde kaydedin; önizleme otomatik yenilenir.
- **Okumayı sade tutar** — araç çubuğu yalnızca üzerine gelince görünür, başlangıç ekranı Dosya Aç ve son kullanılanları sunar.
- **Gerçek Markdown'ı işler** — kod blokları, tablolar, görev listeleri, matematik formülleri, Mermaid diyagramları, görseller, bağlantılar ve yazdırma hepsi çevrimdışı çalışır.

## Yapay Zekâ Kodlama İş Akışlarına Uyar

Araçlarınızın ürettiği belgeler için küçük, önizleme-öncelikli bir çalışma alanı olarak kullanın:

- Claude Code / Codex / Cursor'ın ürettiği planları, görev notlarını ve README'leri koca bir IDE açmadan sekme olarak açık tutun.
- Uygulamayı yeniden başlattıktan sonra aynı sekme sırasına ve aktif belgeye dönün; etkin olmayan dosyalar yalnızca seçildiğinde diskten yüklenir.
- MD Preview içinde küçük kaynak düzenlemeleri yapın, başka bir editör dosyayı yazınca yine de canlı yenileme alın.
- macOS'ta, Finder'dan yeni bir Markdown belgesi oluşturup önce VS Code açmak yerine doğrudan kaynak düzenlemeye inin.
- Temiz bir PDF gerektiğinde render edilmiş önizlemeyi yazdırın veya dışa aktarın.

## İndirme

En son sürümü [GitHub Releases](https://github.com/vorojar/md-preview/releases)'tan alın.

| Platform | Paket | Notlar |
|---|---|---|
| Windows | `MD-Preview-windows-x64.exe` | Tek dosyalık uygulama. Uygulama içi güncelleyici bir sonraki exe'yi indirir, SHA-256 özetini doğrular, kendini değiştirir ve yeniden başlar. |
| Linux | `MD-Preview-linux-x64.tar.gz` | Sistemde WebKitGTK çalışma zamanı gerektirir. |
| Android | `MD-Preview-Android.apk` | Dosyalar, WeChat, WeCom ve paylaşım sayfalarından Markdown açmak için native Android görüntüleyici. |

Android derlemeleri ayrı mobil sürümler olarak yayınlanır, örneğin [mobile-android-v1.0.10](https://github.com/vorojar/md-preview/releases/tag/mobile-android-v1.0.10).

Kaynaktan da derleyebilirsiniz:

```bash
git clone https://github.com/vorojar/md-preview.git
cd md-preview
cargo build --release
./target/release/md-preview README.md
```

## Kullanım

```bash
# Bir veya birden çok dosyayı doğrudan açın
md-preview README.md plan.md task.md

# Ya da boş bir pencere açıp Dosya Aç'ı kullanın, son kullanılanlardan seçin, ya da bir dosya sürükleyin
md-preview
```

MD Preview, `.md` ve `.txt` dosyalarını sürükle-bırak, aç iletişim kutusu, son kullanılanlar veya komut satırı üzerinden kabul eder. Masaüstü belgeleri sekme olarak açılır; aynı yolu tekrar açmak mevcut sekmeyi etkinleştirir. Mevcut belgenin yanında bir Markdown dosyası oluşturup hemen kaynak düzenlemeye girmek için sekme çubuğundaki `+`'ı veya `Cmd/Ctrl+N`'i kullanın. Sekme sırası ve aktif belge her açılışta geri yüklenir, etkin olmayan içerik yalnızca seçilene kadar diskte kalır. Göreli görseller ve desteklenen yerel belge bağlantıları, geçerli Markdown dosyasının dizininden çözümlenir, böylece dokümantasyon klasörleri doğal şekilde render olur ve gezilebilir.

Bir sekmenin dosyası taşınır veya silinirse sekme sessizce kaybolmak yerine görünür kalır. Dosyayı yeniden bulmak veya sekmeyi kapatmak için seçin.

### macOS Finder eylemleri

Noter onaylı macOS uygulaması bir Finder eklentisi içerir. `MD Preview.app`'i Applications'a sürükledikten sonra bir kez açın. macOS eklentiyi otomatik etkinleştirmezse **Sistem Ayarları → Genel → Oturum Açma Öğeleri ve Uzantılar → Finder Uzantıları**'nı kullanın.

Markdown, metin, JSON veya HTML dosyası oluşturmak, klasör yolunu kopyalamak veya klasörü Terminal'de açmak için bir Finder klasörünün içinde sağ tıklayın. **New Markdown**, çakışmayan bir dosya adı oluşturur ve doğrudan MD Preview'in kaynak düzenleyicisinde açar.

iPhone ve iPad'de Local Markdown Preview, Dosyalar ve iOS paylaşım sayfasından Markdown ve düz metin dosyalarını açar. Android'de MD Preview, sistemin "Birlikte aç" ve paylaşım akışlarında Markdown dosyaları için görünür. Son kullanılan dosyalar uygulama içinde özel olarak önbelleğe alınır, böylece WeChat veya WeCom gibi geçici sağlayıcılardan açılan dosyalar daha sonra da erişilebilir kalır; eskimiş son-kullanılan girişleri çökmeden güvenle kaldırılır.

## Özellikler

| Özellik | Ne anlama gelir |
|---|---|
| Masaüstü sekmeleri | Birden fazla Markdown veya metin belgesini tek pencerede açın; yinelenen yollar mevcut sekmeyi etkinleştirir. |
| Oturum geri yükleme | Etkin olmayan belge gövdelerini önbelleğe almadan, yeniden başlatma sonrası sekme sırasını ve aktif belgeyi geri yükler. |
| Kayıp dosyalar | Taşınan veya silinen dosyalar, Bul ve Kapat eylemleriyle açıkça "kayıp" sekme olarak kalır. |
| Finder iş akışı | macOS'ta, Finder'dan Markdown oluşturun ve hemen MD Preview'de düzenlemeye başlayın. |
| Güvenilir otomatik kayıt | Kaynak düzenlemeleri kısa bir duraklamadan sonra kaydedilir; önizleme, sekme değişimi, sekme/pencere kapatma veya çıkıştan önce yazılır; kayıt hataları sekmeyi ve metni bozmadan korur. |
| Yerel belge bağlantıları | Var olan Markdown ve metin dosyalarına göreli veya mutlak bağlantılar bir sekme açar veya etkinleştirir; geçersiz yerel hedefler önizlemenin yerini almaz. |
| Ön bilgi (front matter) | Belge başındaki YAML meta verisi bir başlığa dönüşmek yerine okunabilir meta veri olarak kalır. |
| Canlı istatistikler | Sekme çubuğu, boşluksuz ve toplam karakter sayılarını gösterir, düzenlerken güncellenir. |
| İçerik yakınlaştırma | Sekme çubuğunu veya araç çubuğunu yeniden boyutlandırmadan render edilmiş belgeyi veya kaynak metni %70-%200 arasında yakınlaştırın. |
| Kaydırma sürekliliği | Önizleme ve kaynak düzenleme, yükseklikleri farklı olsa da normalize edilmiş okuma ilerlemesini korur. |
| Başlangıç ekranı | Boş açılışlarda Dosya Aç ve yerel son kullanılanlar gösterilir, böylece uygulama hiçbir şey yüklenmeden önce de kullanışlıdır. |
| Mobil açma | iOS, Dosyalar ve paylaşım sayfasından Markdown açar; Android, Dosyalar, WeChat, WeCom ve Android paylaşım sayfalarından Markdown açabilir. |
| Sürükle bırak | Pencereye bir Markdown dosyası bırakın, hemen açılır. |
| CLI ile açma | `md-preview path/to/file.md` doğrudan bir kabuktan açılır. |
| Önizlemede arama | `Cmd/Ctrl+F`, render edilmiş belge için kompakt bir arama çubuğu açar. |
| Canlı yenileme | Dış düzenlemeler render edilmiş belgeyi otomatik yeniler. |
| Satır içi kaynak düzenleme | `Cmd/Ctrl+E` kaynak moduna geçer; düzenlemeler otomatik kaydedilir, `Cmd/Ctrl+S` anında kayda zorlar. |
| Native yazdırma | `Cmd/Ctrl+P` platformun yazdırma iletişim kutusunu açar ve yalnızca önizlemeyi yazdırır. |
| Syntax highlighting | highlight.js çevrimdışı gömülüdür, ilk boyamadan sonra enjekte edilir. |
| Matematik | KaTeX, `$...$`, `$$...$$`, `\(...\)` ve `\[...\]`'i talep üzerine render eder. |
| Diyagramlar | Belge gerçekten kullandığında Mermaid fenced blokları yerelde render edilir. |
| GitHub Uyarıları | `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]` ve `[!CAUTION]` blockquote'ları uyarı kutucukları olarak render edilir. |
| Vurgulamalar | `==vurgu==`, notlar ve yapay zekâ üretimi belgeler için işaretli metin olarak render edilir. |
| Karanlık mod | macOS, Windows ve Linux'ta sistemin renk şemasını takip eder. |
| GitHub biçimli Markdown | Tablolar, görev listeleri, üstü çizili, başlık öznitelikleri ve anchor'lar. |
| Dış bağlantılar | `http`, `https` ve `mailto` bağlantıları sistem tarayıcısında veya posta uygulamasında açılır. |
| Pencere geri yükleme | Bağlı bir monitörde hâlâ görünürse son boyut ve konum geri yüklenir. |
| Güncellemeler | İlk boyamadan sonra MD Preview masaüstü GitHub Releases'i kontrol eder. macOS imzalı uygulama-içi güncellemeler için Sparkle kullanır; Windows SHA-256 doğrulamasından sonra tek exe'yi kendisi günceller; Linux eşleşen sürüm indirmesini açar. |

## Klavye Kısayolları

| Kısayol | Eylem |
|---|---|
| `Cmd/Ctrl + N` | Markdown dosyası oluştur ve kaynak düzenlemeye gir |
| `Cmd/Ctrl + O` | Dosya aç |
| `Cmd/Ctrl + F` | Önizlemede ara |
| `Cmd/Ctrl + E` | Önizleme/kaynak düzenleme arasında geçiş yap |
| `Cmd/Ctrl + S` | Kaynak düzenleme modunda kaydet |
| `Cmd/Ctrl + P` | Önizlemeyi yazdır |
| `Cmd/Ctrl + W` | Aktif sekmeyi kapat; hiç belge sekmesi kalmayınca pencereyi kapat |
| `Cmd/Ctrl +` | Belge içeriğini yakınlaştır |
| `Cmd/Ctrl -` | Belge içeriğini uzaklaştır |
| `Cmd/Ctrl 0` | Belge içeriği yakınlaştırmasını sıfırla |
| `Esc` | Kaynak düzenleme modundan çık, gerekirse kaydet |

## Markdown Desteği

MD Preview, temel Markdown geçişi için `pulldown-cmark` kullanır, sonra render edilen belgeyi yalnızca gerektiğinde zenginleştirir:

- CommonMark artı GFM tarzı tablolar, görev listeleri, üstü çizili ve başlık öznitelikleri
- Notlar, ipuçları, uyarılar için GitHub tarzı alert blockquote'ları
- Birçok Markdown not aracının kullandığı `==vurgu==` metin işaretleri
- Delphi/Pascal dahil 40'tan fazla dil için çevrimdışı kod renklendirme
- Markdown vurgusunun formülleri bozmaması için güvenlik önlemli, çevrimdışı KaTeX matematik render'ı
- Fenced ` ```mermaid ` blokları için çevrimdışı Mermaid render'ı
- Dosya başına `<base>` URL'siyle göreli görsel yolları
- Desteklenen yerel Markdown veya metin belgelerine göreli ve mutlak bağlantılar
- `---` veya `...` ile sınırlanan okunabilir YAML ön bilgisi (front matter)
- Uygulama kontrollerini çıktıdan kaldıran yazdırma CSS'i

Soğuk yol küçük kalır: sıradan Markdown önce render edilir, highlight.js, KaTeX ve Mermaid gibi daha ağır zenginleştiriciler ilk görünür boyamadan sonraya ertelenir veya yalnızca ihtiyaç duyan belgeler için yüklenir.

## Neden Küçük Kalıyor

MD Preview bir Tauri veya Electron uygulaması değil. Şunları kullanır:

- Native kabuk ve Markdown hattı için **Rust**
- Sistemin WebView'i için **wry**: macOS'ta WebKit, Windows'ta WebView2, Linux'ta WebKitGTK
- Cross-platform pencere/olay döngüsü için **tao**
- Markdown ayrıştırma için **pulldown-cmark**
- Dosya izleme için **notify**
- Native aç iletişim kutuları için **rfd**

Release profili boyut odaklı optimizasyonu, LTO'yu, tek codegen unit'i, sembol temizlemeyi ve `panic = "abort"`'u etkinleştirir.

## Gizlilik

MD Preview'in hesabı, telemetrisi veya analitiği yoktur. Markdown dosyalarınız diskte kalır. Render işlemi yerelde gerçekleşir. Masaüstü uygulamasının kendisinin yaptığı tek ağ isteği, ilk boyamadan sonraki isteğe bağlı güncelleme kontrolüdür; başarısız kontroller yok sayılır ve başlangıcı asla engellemez. macOS güncellemeleri, uygulamanın gömülü EdDSA genel anahtarını kullanarak Sparkle tarafından doğrulanır. Windows kendi kendini güncellerken, çalışan exe'yi değiştirmeden önce GitHub Releases'in döndürdüğü SHA-256 özetini doğrular.

## Sorun Giderme

**Linux'ta açılmıyor**

Dağıtımınız için WebKitGTK 4.1 paketlerini kurun. Debian/Ubuntu'da:

```bash
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
```

**Linux'ta NVIDIA'da boş pencere açılıyor**

MD Preview, NVIDIA sürücüsü yüklü Linux sistemlerinde otomatik olarak korumacı bir WebKitGTK yedek ayarı uygular. Dağıtımınız hâlâ boş bir WebView gösteriyorsa şunu elle deneyin:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 md-preview your-file.md
```

Bu işe yaramazsa şunu deneyin:

```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 md-preview your-file.md
```

**Windows, MD Preview'i varsayılan uygulama olarak otomatik ayarlayamıyor**

Windows, uygulamaların dosya ilişkilendirmelerini sessizce devralmasına izin vermez. MD Preview kendini "Birlikte aç" listesine kaydeder; Explorer veya Windows Ayarları'ndan seçin.

**Bir formül veya diyagram metin olarak görünüyor**

Söz diziminin geçerli Markdown/KaTeX/Mermaid olduğundan emin olun. Matematik ve Mermaid talep üzerine yüklenir, bu yüzden bu kalıpları içermeyen belgeler başlangıç maliyetini ödemez.

## Geliştirme

```bash
cargo build
cargo test
cargo build --release
```

CI ve release derlemeleri Windows ve Linux'u kapsar. Android ayrı yayınlanır.

Bakımcı release akışı:

```bash
scripts/release.sh v1.2.3
```

Script doğrulamayı çalıştırır, `master`'ı ve tag'i push eder, GitHub Actions'ı bekler, Windows ve Linux varlıklarını doğrular.

## Lisans

[MIT](LICENSE)
