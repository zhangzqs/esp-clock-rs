#!/usr/bin/env python3

import struct
import sys


def ReadUntil(f, buf):
    buf = bytes(buf)
    if not len(buf):
        return
    while True:
        t = f.read(1)
        if not t:
            raise EOFError
        if buf.startswith(t):
            pos = f.tell()
            if f.read(len(buf) - 1) == buf[1:]:
                return
            f.seek(pos, 0)


def ReadOrEOF(f, size):
    b = f.read(size)
    if len(b) == size:
        return b
    else:
        raise EOFError


def WriteAll(f, buf):
    if f.write(buf) != len(buf):
        raise IOError('Error while writing file')


def ScanBigInt(f, prefix=b''):
    res = 0
    count = -len(prefix)
    while True:
        if prefix:
            t = struct.unpack('>B', prefix[0:1])[0]
            prefix = prefix[1:]
        else:
            t = struct.unpack('>B', ReadOrEOF(f, 1))[0]
        count += 1
        if t & 0x80:
            res = (res << 7) | (t & 0x7f)
        else:
            return ((res << 7) | t, count)


def PrintBigInt(n):
    res = struct.pack('>B', n & 0x7f)
    while n >= 0x80:
        n >>= 7
        res = struct.pack('>B', (n & 0x7f) | 0x80) + res
    return res


def main(fnin, fnout):
    fin = open(fnin, "rb")
    ReadUntil(fin, b'MThd')
    mthd_len = struct.unpack('>L', ReadOrEOF(fin, 4))[0]
    assert mthd_len == 6
    mthd = struct.unpack('>HHH', ReadOrEOF(fin, 6))
    assert mthd[0] == 1
    events = []
    last_no = 0
    max_time = 0
    have_title = False
    for trkno in range(mthd[1]):
        mtrk = ReadOrEOF(fin, 4)
        assert mtrk == b'MTrk'
        mtrk_len = struct.unpack('>L', ReadOrEOF(fin, 4))[0]
        mtrk_len_read = 0
        last_time = 0
        last_meta = 0x80
        while mtrk_len_read < mtrk_len:
            timestamp = ScanBigInt(fin)
            mtrk_len_read += timestamp[1]
            if mtrk_len_read >= mtrk_len:
                raise EOFError
            last_time += timestamp[0]
            if last_time > max_time:
                max_time = last_time
            meta = ReadOrEOF(fin, 1)
            mtrk_len_read += 1
            if mtrk_len_read >= mtrk_len:
                raise EOFError
            meta_int = struct.unpack('>B', meta)[0]
            if meta_int & 0x80:
                last_meta, param = meta, ReadOrEOF(fin, 1)
                mtrk_len_read += 1
                if mtrk_len_read > mtrk_len:
                    raise EOFError
            else:
                meta, param = last_meta, meta
                meta_int = struct.unpack('>B', meta)[0]
            if meta_int == 0xff:
                meta_len = ScanBigInt(fin)
                mtrk_len_read += meta_len[1]
                if mtrk_len_read > mtrk_len:
                    raise EOFError
                param += PrintBigInt(meta_len[0])
                meta_data = ReadOrEOF(fin, meta_len[0])
                mtrk_len_read += meta_len[0]
                if mtrk_len_read > mtrk_len:
                    raise EOFError
                if param[0:1] == b'\x03' and not have_title:
                    have_title = True
                    events.append((last_time, last_no, meta, param, meta_data))
                elif param[0:1] not in (b'\x03', b'\x04', b'\x2f'):
                    events.append((last_time, last_no, meta, param, meta_data))
            elif meta_int & 0xf0 == 0xf0:
                meta_len = ScanBigInt(fin, param)
                mtrk_len_read += meta_len[1]
                if mtrk_len_read > mtrk_len:
                    raise EOFError
                meta_data = ReadOrEOF(fin, meta_len[0])
                mtrk_len_read += meta_len[0]
                if mtrk_len_read > mtrk_len:
                    raise EOFError
                events.append((last_time, last_no, meta, PrintBigInt(meta_len[0]), meta_data))
            else:
                if meta_int >> 4 not in (0xc, 0xd):
                    param += ReadOrEOF(fin, 1)
                    mtrk_len_read += 1
                    if mtrk_len_read > mtrk_len:
                        raise EOFError
                events.append((last_time, last_no, meta, param, b''))
            last_no += 1
    fin.close()
    events.sort()
    fout = open(fnout, 'wb')
    WriteAll(fout, struct.pack('>4sLHHH4sL', b'MThd', 6, 0, 1, mthd[2], b'MTrk', 0))
    write_len = 0
    last_time = 0
    last_meta = b'\x00'
    for event in events:
        timestamp = event[0] - last_time
        last_time = event[0]
        assert timestamp >= 0
        if struct.unpack('>B', last_meta)[0] & 0xf0 != 0xf0 and last_meta == event[2]:
            write_buf = PrintBigInt(timestamp) + event[3] + event[4]
        else:
            write_buf = PrintBigInt(timestamp) + event[2] + event[3] + event[4]
            last_meta = event[2]
        write_len += len(write_buf)
        WriteAll(fout, write_buf)
    timestamp = max_time - last_time
    if timestamp < 0:
        timestamp = 0
    write_buf = PrintBigInt(timestamp) + b'\xff\x2f\x00'
    write_len += len(write_buf)
    WriteAll(fout, write_buf)
    fout.seek(18)
    WriteAll(fout, struct.pack('>L', write_len))
    fout.close()

if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.stderr.write('Usage: %s input.mid output.mid\n\n' % sys.argv[0])
        sys.exit(1)
    else:
        try:
            sys.exit(main(sys.argv[1], sys.argv[2]))
        except Exception as e:
            raise
            sys.stderr.write('Error: %s %s\n' % (type(e).__name__, e))