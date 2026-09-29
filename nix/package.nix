{
  self,
  pkgs,
  ...
}:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "sgit";
  version = "0.2.0";

  src = "${self}";
  pwd = "${self}";

  # Keep Cargo.lock authoritative so nix does not need a cargoHash.
  cargoLock = {
    lockFile = "${src}/Cargo.lock";
  };

  nativeBuildInputs = with pkgs; [
    pkg-config
    git
  ];

  buildInputs = with pkgs; [
    openssl.dev
    libgit2.dev
    zlib
  ];

  doCheck = false;

  meta = with pkgs.lib; {
    description = "Manage projects with (nested) git submodules";
    homepage = "https://github.com/vhdirk/sgit";
    license = licenses.unfree;
    mainProgram = "sgit";
    platforms = platforms.unix;
    maintainers = [ ];
  };
}
