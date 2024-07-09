{pkgs, ...}: {
  # https://devenv.sh/languages/
  languages.nix.enable = true;
  languages.rust.enable = true;

  # https://devenv.sh/packages/
  packages = with pkgs; [
    alejandra
    otel-cli
    cargo-rdme
  ];

  pre-commit = {
    # https://devenv.sh/pre-commit-hooks/
    hooks = {
      alejandra.enable = true;
      cargo-check.enable = true;
      clippy.enable = true;
      clippy.settings.denyWarnings = true;
      rustfmt.enable = true;
      taplo.enable = true;
    };
  };

  env = {
    OTEL_EXPORTER_OTLP_ENDPOINT = "grpc://localhost:4317";
    OTEL_EXPORTER_OTLP_INSECURE = "true";
  };
  # See full reference at https://devenv.sh/reference/options/
}
