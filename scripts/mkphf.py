import csv
import re
import sys

origin_re = re.compile(r"\s*ORIGIN\s*=\s*(0X[0-9A-Fa-f]*)\s*")
length_re = re.compile(r"\s*LENGTH\s*=\s*(0X[0-9A-Fa-f]*)\s*")
sectors_re = re.compile(r"\s*SIZE\s*[0-9]*\s*AS\s*[0-9]*\s*([0-9]*)-BYTE\s*SEGMENTS\s*")  # noqa: E501


def main():
    debug_names = set()
    with open(sys.argv[1]) as fp:
        for msp in fp.read().splitlines():
            debug_names.add(msp)

    not_preset_in_debug = set()
    used_debug = set()
    info = dict()

    with open(sys.argv[2], newline='') as csvfile:
        mspreader = csv.reader(csvfile, delimiter=',')
        while True:
            try:
                (name, origin, length, end, sectors) = next(mspreader)
            except ValueError:
                continue
            except StopIteration:
                break

            if name not in debug_names:
                not_preset_in_debug.add(name)
            else:
                info[name] = extract_info(origin, length, sectors)
                used_debug.add(name)

    not_present_in_headers = debug_names - used_debug
    manual_rules = dict()

    for n in not_present_in_headers:
        res = manual_override(n)
        if res:
            manual_rules[n] = res

    not_present_in_headers -= set(manual_rules)

    print("""use phf::{phf_map, Map};

/* Autogenerated by mkphf.py */
pub(crate) static INFOMEM_MAP: Map<&'static str, Option<(u16, u16, u16)>> = phf_map! {""")  # noqa: E501
    print("/* Autogenerated from headers. */")
    for n, (o, l, s) in info.items():
        if n:
            print(f"\"{n}\" => Some((0x{o:X}, 0x{l:X}, {s})),")

    print("/* Manual override given. */")
    for n, (o, l, s) in manual_rules.items():
        if n:
            print(f"\"{n}\" => Some((0x{o:X}, 0x{l:X}, {s})),")

    print("/* Could not be extract from headers (and no manual override given). */")  # noqa: E501
    for n in not_present_in_headers:
        if n:
            print(f"\"{n}\" => None,")
    # for n in not_preset_in_debug:
    #     print(f"\"{n}\" => None")

    print("};")


def extract_info(origin, length, sectors):
    origin = int(origin_re.match(origin)[1], base=16)
    length = int(length_re.match(length)[1], base=16)
    sector_size = int(sectors_re.match(sectors)[1])

    return (origin, length, sector_size)


def manual_override(name):
    match name:
        case "F20x1_G2x0x_G2x1x" | "F20x2_G2x2x_G2x3x" | "MSP430G2xx2" | \
             "MSP430G2xx3":
            return (0x1000, 0x100, 64)
        case _:
            return None


if __name__ == "__main__":
    main()
