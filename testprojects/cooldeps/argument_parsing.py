import argparse

def parse_cli():
  # Parse the command line arguments
  parser = argparse.ArgumentParser()
  parser.add_argument("--enable-strict-deps", action="store_true", dest="enable_strict_deps")
  parser.add_argument("dependencies_file")
  parser.add_argument("direct_dependees_file")
  parser.add_argument("target")
  return parser.parse_args()

class tcolors:
  HEADER = '\033[95m'
  OKBLUE = '\033[94m'
  OKGREEN = '\033[92m'
  WARNING = '\033[93m'
  FAIL = '\033[91m'
  ENDC = '\033[0m'
  BOLD = '\033[1m'
  UNDERLINE = '\033[4m'
