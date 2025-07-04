{
  busybox-sandbox-shell,
  lib,
  makeWrapper,
  naersk,
  pkg-config,
  ...
}:
naersk.buildPackage rec {
  name = "rust_pty";
  src = ./.;
  buildInputs = [
    busybox-sandbox-shell
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
