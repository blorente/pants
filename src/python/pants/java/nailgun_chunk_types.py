# Copyright 2019 Pants project contributors (see CONTRIBUTORS.md).
# Licensed under the Apache License, Version 2.0 (see LICENSE).


class ChunkType:
  """Nailgun protocol chunk types.

  N.B. Because we force `__future__.unicode_literals` in sources, string literals are automatically
  converted to unicode for us (e.g. 'xyz' automatically becomes u'xyz'). In the case of protocol
  implementations, supporting methods like struct.pack() require ASCII values - so we specify
  constants such as these as byte literals (e.g. b'xyz', which can only contain ASCII values)
  rather than their unicode counterparts. The alternative is to call str.encode('ascii') to
  convert the unicode string literals to ascii before use.
  """

  ARGUMENT = b'A'
  ENVIRONMENT = b'E'
  WORKING_DIR = b'D'
  COMMAND = b'C'
  STDIN = b'0'
  STDOUT = b'1'
  STDERR = b'2'
  START_READING_INPUT = b'S'
  STDIN_EOF = b'.'
  EXIT = b'X'

  @classmethod
  def REQUEST_TYPES(cls): return (cls.ARGUMENT, cls.ENVIRONMENT, cls.WORKING_DIR, cls.COMMAND)

  @classmethod
  def EXECUTION_TYPES(cls): return (cls.STDIN, cls.STDOUT, cls.STDERR, cls.START_READING_INPUT, cls.STDIN_EOF, cls.EXIT)

  @classmethod
  def VALID_TYPES(cls): return cls.REQUEST_TYPES() + cls.EXECUTION_TYPES()


class PailgunChunkType(ChunkType):
  # PGRP and PID are custom extensions to the Nailgun protocol spec for transmitting pid info.
  # PGRP is used to allow the client process to try killing the nailgun server and everything in its
  # process group when the thin client receives a signal. PID is used to retrieve logs for fatal
  # errors from the remote process at that PID.
  # TODO(#6579): we should probably move our custom extensions to a ChunkType subclass in
  # nailgun_client.py and differentiate clearly whether the client accepts the pailgun extensions
  # (e.g. by calling it PailgunClient).
  PGRP = b'G'
  PID = b'P'

  # NB: Override the parent method, to add PGRP and PID chunks.
  @classmethod
  def EXECUTION_TYPES(cls): return super().EXECUTION_TYPES() + (cls.PGRP, cls.PID)


# Mixins to define which chunk types you'll have access to from cls.ChunkType
class ChunkTypeMixin:
  ChunkType = ChunkType


class PailgunChunkTypeMixin(ChunkTypeMixin):
  ChunkType = PailgunChunkType
