# Maintainer: smtdfc smtdfc@bakeryos.smtdfc.net
pkgname=bakeryos-dev-wizard
pkgver=1.0.0
pkgrel=1
pkgdesc="Dev Wizard for BakeryOS"
arch=('x86_64')
url="https://github.com/bakeryos-project/bakeryos-dev-wizard"
license=('MIT')
depends=('gtk4' 'glib2')
makedepends=('meson' 'ninja' 'rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    arch-meson . build
    ninja -C build
}

package() {
    cd "$pkgname-$pkgver"
    DESTDIR="$pkgdir" ninja -C build install
}
