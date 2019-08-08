# Copyright 2019 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).

import unittest

from pants.java.nailgun_chunk_types import ChunkType


class TestChunkType(unittest.TestCase):
  def test_chunktype_constants(self):
    self.assertIsNotNone(ChunkType.ARGUMENT)
    self.assertIsNotNone(ChunkType.ENVIRONMENT)
    self.assertIsNotNone(ChunkType.WORKING_DIR)
    self.assertIsNotNone(ChunkType.COMMAND)
    self.assertIsNotNone(ChunkType.STDIN)
    self.assertIsNotNone(ChunkType.STDOUT)
    self.assertIsNotNone(ChunkType.STDERR)
    self.assertIsNotNone(ChunkType.START_READING_INPUT)
    self.assertIsNotNone(ChunkType.STDIN_EOF)
