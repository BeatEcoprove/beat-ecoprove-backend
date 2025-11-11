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
        containerTool = "podman";
        pkgs = nixpkgs.legacyPackages.${system};
        servicesConfig = import ./services.nix;

        buildService = name: config:
          if config.flake then
            let
              img = inputs.${name}.packages.${system}.docker;
            in
            pkgs.writeShellScript "build-${name}" ''
              echo "📦 Loading ${name}..."
              ${containerTool} load < ${img}
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

              VERSION=$(nix eval --raw "${src}#version.${system}" 2>/dev/null || echo "latest")

              ${containerTool} build -t ${name}:$VERSION .
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
