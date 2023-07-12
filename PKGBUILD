pkgname="juodas-calc"  
pkgver="alpha_0.0.1"
makedepends=("rust>=1.65.0")
pkgrel="1"
pkgdesc="Simple calculator"
arch=("x86_64")
license=("MIT")

#source=("juodas-calc-src"::"file://${PWD}/../../src/"
#        "juodas-calc-cargo-cfg"::"file://${PWD}/../../Cargo.toml"

source=(
        #"src"
        "Cargo.toml"
        "favicon.png"
        "juodas-calc.desktop")
sha512sums=(
            #"SKIP"
            "SKIP"
            "SKIP"
            "SKIP")

package() {
    mkdir ${srcdir}/compilation
    cp -R ${srcdir}/../src ${srcdir}/compilation/src
    cp ${srcdir}/Cargo.toml ${srcdir}/compilation 
    cd ${srcdir}/compilation
    cargo build --release
    cd ..
    install -D "${srcdir}/compilation/target/release/juodas-calc" \
               "${pkgdir}/usr/bin/juodas-calc" 
    install -D "${srcdir}/favicon.png" \
               "${pkgdir}/usr/share/icons/hicolor/48x48/apps/juodas-calc.png"
    install -D "${srcdir}/juodas-calc.desktop" \
               "${pkgdir}/usr/share/applications/juodas-calc.desktop"
    chmod +x "${pkgdir}/usr/bin/juodas-calc" 
}
