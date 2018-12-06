import sys
import subprocess

list_file = sys.argv[1]
max_args = 2500

with open(list_file) as f:
  all_targets = [target.strip("\n") for target in f.readlines() if not target.startswith("//:") and not target.startswith("3rdparty")]

  print("All targets = {}".format(len(all_targets)))

  for i in range(0, int(len(all_targets) / max_args)):
    chunk_start = i*max_args
    chunk_end = chunk_start + max_args
    args = all_targets[chunk_start:chunk_end]
    print("Running pants with targets [{}, {}]... len(args) = {}, e.g. {}".format(chunk_start, chunk_end, len(args), args[49]))
    command = [
      "./pants",
      "clean-all",
      "dep-usage.jvm",
      "--no-cache-compile-zinc-read",
      "--no-summary",
      "--no-transitive",
      "--output-file=../misc/transitive-live-video-data-" + str(i) + ".json"
    ] + args
    subprocess.run(command)
    print("DONE")
