{
  pkgs ? import <nixpkgs> {
    config = {
      allowUnfree = true;
    };
  },
}:

pkgs.mkShell {
  ANDROID_SDK_ROOT = "/home/tau2c/Android/Sdk";
  packages = with pkgs; [
    flutter
    android-tools
    pkg-config
    
    # flutter_rust_bridge_codegen

    sqlite

    rustup

    nixfmt-rfc-style

    sqlite-web
    
    nodejs
  ]
  ++ (if pkgs.stdenv.isx86_64 then [android-studio bruno] else [])
  ++ (if pkgs.stdenv.isAarch64 then [] else []);
  
  shellHook = ''
    export LD_LIBRARY_PATH=build/linux/x64/debug/bundle/lib:$LD_LIBRARY_PATH
    export PATH=$HOME/.cargo/bin:$PATH
  '';
}
