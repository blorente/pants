import argparse

def parse_cli():
  # Parse the command line arguments
  parser = argparse.ArgumentParser()
  parser.add_argument("--enable-strict-deps", action="store_true", dest="enable_strict_deps")
  parser.add_argument("dependencies_file")
  parser.add_argument("direct_dependees_file")
  parser.add_argument("target")
  return parser.parse_args()
