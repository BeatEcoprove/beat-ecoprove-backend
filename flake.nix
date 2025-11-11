{
  description = "Beat Ecoprove Backend";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    auth-service.url = "github:BeatEcoprove/beat-ecoprove-auth";
    core-service.url = "github:BeatEcoprove/beat-ecoprove-core";
    messaging-service.url = "github:BeatEcoprove/beat-ecoprove-messaging";
  };

  outputs = { self, nixpkgs, flake-utils, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        containerTool = let
        envVal = builtins.getEnv "CONTAINER_TOOL";
        in
        if envVal == "" then "podman" else envVal;

        pkgs = nixpkgs.legacyPackages.${system};
        servicesConfig = import ./nix/services.nix;

        systemLinux =
          if pkgs.stdenv.isAarch64 then "aarch64-linux"
          else if pkgs.stdenv.isx86_64 then "x86_64-linux"
          else system;

        buildService = name: config:
          if config.flake then
            let
              img = inputs.${name}.packages.${systemLinux}.docker;
            in
            pkgs.writeShellScript "build-${name}" ''
              echo "📦 Loading ${name}..."
              ${containerTool} load < ${img}

              ${containerTool} tag localhost/${name}:$VERSION ${name}:$VERSION 2>/dev/null || true
              ${containerTool} tag ${name}:$VERSION localhost/${name}:$VERSION 2>/dev/null || true

              echo "✅ Loaded ${name}"
            ''
          else
            let
              src = inputs.${name};
            in
            pkgs.writeShellScript "build-${name}" ''
              set -e
              echo "🔨 Building ${name}..."
              TEMP_DIR=$(mktemp -d)
              trap "chmod -R u+w $TEMP_DIR 2>/dev/null || true; rm -rf $TEMP_DIR" EXIT
              cp -r ${src}/* $TEMP_DIR/
              cd $TEMP_DIR

              VERSION=$(nix eval --raw "${src}#version.${systemLinux}" 2>/dev/null || echo "latest")

              ${containerTool} build -t ${name}:$VERSION .
              ${containerTool} tag ${name}:$VERSION localhost/${name}:$VERSION 2>/dev/null || true

              echo "✅ Built ${name}"
            '';
      in
      {
        apps = (builtins.mapAttrs (name: config: {
          type = "app";
          program = toString (buildService name config);
        }) servicesConfig) // {
          build-all = {
            type = "app";
            program = toString (pkgs.writeShellScript "build-all" ''
              set -e
              echo "🚀 Building all Beat EcoProve services..."
              echo ""
              ${pkgs.lib.concatMapStringsSep "\n"
                (name: "${buildService name servicesConfig.${name}}")
                (builtins.attrNames servicesConfig)}
              echo ""
              echo "✅ All services built!"
              echo ""
              '');
            };
          };
      });
}
