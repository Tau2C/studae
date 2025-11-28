{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  packages = with pkgs; [
    flutter
    
    # flutter_rust_bridge_codegen

    rustup

    nixfmt-rfc-style
    
    nodejs
  ];
  
  shellHook = ''
    export PATH=$HOME/.cargo/bin:$PATH
  '';
}
