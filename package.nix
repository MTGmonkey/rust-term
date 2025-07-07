{
  busybox-sandbox-shell,
  expat,
  fontconfig,
  freetype,
  lib,
  libGL,
  libxkbcommon,
  makeWrapper,
  naersk,
  pkg-config,
  wayland,
  xorg,
  ...
}:
naersk.buildPackage rec {
  name = "rust_term";
  src = ./.;
  buildInputs = [
    busybox-sandbox-shell
    expat
    fontconfig
    freetype
    freetype.dev
    libGL
    libxkbcommon
    wayland
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];
  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];
  postInstall = ''
    wrapProgram "$out/bin/${meta.mainProgram}" --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath buildInputs}"
  '';
  meta = {
    mainProgram = "rust_term";
    homepage = "https://mtgmonkey.net";
  };
}
