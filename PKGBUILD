# Maintainer: Your Name <your.email@example.com>

pkgname=brain
pkgver=0.1.0
pkgrel=1
pkgdesc="A CLI tool for querying your knowledge base with LLM integration"
arch=('x86_64' 'aarch64')
url="https://github.com/xorphitus/brain"
license=('MIT')
depends=('gcc-libs' 'ollama')
makedepends=('rust' 'cargo' 'git')
optdepends=('emacs: For brain-search.el integration'
            'jq: Required for brain-search.el')
source=("git+$url.git")
sha256sums=('SKIP')

build() {
  cd "$pkgname"
  cargo build --release
}

check() {
  cd "$pkgname"
  cargo test --release
}

package() {
  cd "$pkgname"
  
  # Install the binary
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  
  # Install the Emacs package
  install -Dm644 "brain-search.el" "$pkgdir/usr/share/emacs/site-lisp/brain-search.el"
  
  # Install license
  install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
