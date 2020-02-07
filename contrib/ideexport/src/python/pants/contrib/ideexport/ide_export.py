# Copyright 2017 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).

import logging
import subprocess
from enum import Enum

from pants.base.workunit import WorkUnitLabel
from pants.task.task import Task
from pants.base.build_environment import get_buildroot
from pants.util.enums import match


logger = logging.getLogger(__name__)


class IdeExport(Task):

  options_scope = 'ide-export'

  class SupportedEditors(Enum):
    intellij = 'intellij'
    vscode = 'vscode'
    none = 'none'

  @classmethod
  def subsystem_dependencies(cls):
    return super().subsystem_dependencies()

  @classmethod
  def register_options(cls, register):
    register(
      '--workspace',
      type=str,
      default=get_buildroot(),
      help='The directory containing the pants build, defaults to the working directory.'
    )
    register(
      '--out',
      type=str,
      default=get_buildroot(),
      help='The directory containing the generated Bloop JSON files.'
    )
    register(
      '--editor',
      type=cls.SupportedEditors,
      default=cls.SupportedEditors.none,
      help='Editor to export to.',
    )

  def _editor_flags(self, options):
    return match(options.editor, {
      self.SupportedEditors.intellij: ['--intellij'],
      self.SupportedEditors.vscode: ['--vscode'],
      self.SupportedEditors.none: [],
    })

  def _run_fastpass(self, target_roots, options):
    cmd_line = [ 'fastpass',
                 '--workspace', options.workspace,
                 '--out', options.out,
               ] + \
               self._editor_flags(options) + \
               [t.address.spec for t in target_roots]

    with self.context.new_workunit('fastpass', labels=[WorkUnitLabel.TOOL]):
      self.context.release_lock()
      subprocess.run(cmd_line)
      self.context.acquire_lock()

  def _test_with_bloop(self, target_roots):
    for target_root in target_roots:
      address = target_root.address.spec
      cmd_line = ['bloop', 'test', address]
      with self.context.new_workunit(f'bloop-{address}', labels=[WorkUnitLabel.TEST, WorkUnitLabel.TOOL]):
        subprocess.run(cmd_line, check=True)

  def execute(self):
    target_roots = self.context.target_roots
    options = self.get_options()
    self._run_fastpass(target_roots, options)
    self._test_with_bloop(target_roots)




