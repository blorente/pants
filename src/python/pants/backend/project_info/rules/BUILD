python_library(
  dependencies=[
    '3rdparty/python:dataclasses',
    'src/python/pants/engine:console',
    'src/python/pants/engine:fs',
    'src/python/pants/engine:objects',
    'src/python/pants/engine:rules',
    'src/python/pants/engine:selectors',
    'src/python/pants/engine/legacy:graph',
    'src/python/pants/rules/core',
    'src/python/pants/subsystem',
    'src/python/pants/util:collections',
    'src/python/pants/util:enums',
    'src/python/pants/util:memo',
  ],
  tags = {"partially_type_checked"},
)

python_tests(
  name='tests',
  dependencies=[
    ':rules',
    'src/python/pants/testutil:goal_rule_test_base',
  ],
  tags = {'partially_type_checked'},
)
