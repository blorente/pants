# Copyright 2017 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).

contrib_plugin(
  name='plugin',
  dependencies=[
    ':ide_export'
  ],
  distribution_name='pantsbuild.pants.contrib.ide_export',
  description='TODO',
  register_goals=True,
)

python_library(
  dependencies = [
    'src/python/pants/base:build_environment',
    'src/python/pants/base:exceptions',
  ],
  tags = {"partially_type_checked"},
)
