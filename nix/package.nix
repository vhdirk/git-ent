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
    makeWrapper
  ];

  buildInputs = [];

  postInstall = ''
    wrapProgram $out/bin/sgit \
      --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.git ]}
  '';

  doCheck = false;

  meta = with pkgs.lib; {
    description = "Manage projects with (nested) git submodules";
    homepage = "https://github.com/vhdirk/sgit";
    license = licenses.mit;
    mainProgram = "sgit";
    platforms = platforms.unix;
    maintainers = [ ];
  };
}
