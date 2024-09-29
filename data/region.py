import math
from datetime import datetime
from enum import Enum


class RegionType(Enum):
    province = 1  # 省
    z_province = 2  # 自治区
    city = 3  # 市
    district = 4  # 区
    county = 5  # 县
    z_county = 6  # 自治县
    qi = 7  # 旗
    meng = 8  # 盟
    z_city = 9  # 州
    zz_city = 10  # 自治州
    zang_zz_city = 11  # 藏族自治州
    man_zz_county = 12  # 满族自治县
    menggu_zz_county = 13  # 蒙古族自治县
    miao_zz_county = 14  # 苗族自治县
    tu_zz_county = 15  # 土家族自治县
    other = 0  # 其他

    @property
    def label(self) -> str:
        if self == RegionType.province:
            return '省'
        elif self == RegionType.z_province:
            return '自治区'
        elif self == RegionType.city:
            return '市'
        elif self == RegionType.district:
            return '区'
        elif self == RegionType.county:
            return '县'
        elif self == RegionType.z_county:
            return '自治县'
        elif self == RegionType.qi:
            return '旗'
        elif self == RegionType.meng:
            return '盟'
        elif self == RegionType.z_city:
            return '州'
        elif self == RegionType.zz_city:
            return '自治州'
        elif self == RegionType.tu_zz_county:
            return '土家族自治县'
        elif self == RegionType.man_zz_county:
            return '苗族自治县'
        elif self == RegionType.menggu_zz_county:
            return '蒙古族自治县'
        elif self == RegionType.man_zz_county:
            return '满族自治县'
        elif self == RegionType.zang_zz_city:
            return '藏族自治州'
        return ''

    @staticmethod
    def name_classifiction(name: str) -> tuple[str, 'RegionType']:
        if name.endswith('土家族自治县'):
            return name[:-6], RegionType.tu_zz_county
        elif name.endswith('苗族自治县'):
            return name[:-5], RegionType.miao_zz_county
        elif name.endswith('蒙古族自治县'):
            return name[:-6], RegionType.menggu_zz_county
        elif name.endswith('满族自治县'):
            return name[:-5], RegionType.man_zz_county
        elif name.endswith('藏族自治州'):
            return name[:-5], RegionType.zang_zz_city
        elif name.endswith('自治区'):
            return name[:-3], RegionType.z_province
        elif name.endswith('自治县'):
            return name[:-3], RegionType.z_county
        elif name.endswith('自治州'):
            return name[:-3], RegionType.zz_city
        elif name.endswith('省'):
            return name[:-1], RegionType.province
        elif name.endswith('市'):
            return name[:-1], RegionType.city
        elif name.endswith('县'):
            return name[:-1], RegionType.county
        elif name.endswith('区'):
            return name[:-1], RegionType.district
        elif name.endswith('盟'):
            return name[:-1], RegionType.meng
        elif name.endswith('州'):
            return name[:-1], RegionType.z_city
        elif name.endswith('旗'):
            return name[:-1], RegionType.qi
        return name, RegionType.other


class RegionCtr:
    def __init__(self, file_name: str = 'region.dat') -> None:
        self.file_name = file_name

    def pack(self, data_list: list[tuple[str, str, str]] | list[tuple[str, str]], version: int = 0) -> bool:
        version = version or int(datetime.now().strftime('%Y%m%d%H'))
        print('version: ', version)
        with open(self.file_name, 'wb') as f:
            # 写32位版本号
            f.write(version.to_bytes(length=4))
            # 先跳过偏移
            f.seek(6)
            # 写数据, code [i:3] type [i: 1] region [c: n] \n
            index_offset = 6
            offset_map: dict[int, int] = {}
            chars = set()
            for x in data_list:
                name, _ = RegionType.name_classifiction(x[1].replace('*', ''))
                for c in name:
                    chars.add(c)
            char_list: list[str] = list(chars)
            # 字符映射
            char_map: dict[str, int] = {}
            for i, c in enumerate(char_list):
                char_map[c] = i + 64
            for data in data_list:
                discard_year = ''
                if len(data) == 3:
                    code, name, discard_year = data
                else:
                    code, name = data
                code_2 = int(code[:2])
                if code_2 not in offset_map:
                    offset_map[code_2] = index_offset
                name, region_type = RegionType.name_classifiction(name.replace('*', ''))
                # 将名称映射成int列表
                name_char_index_list = []
                for x in name:
                    name_char_index_list.append(char_map[x])
                # 写废止年份，值为 int(discard_year) - 1980， 占6bit
                discard_year_int = 0
                if discard_year:
                    discard_year_int = int(discard_year) - 1980
                total_bits = 8 + 20 + 4 + 12 * len(name_char_index_list)
                if discard_year_int:
                    total_bits += 6
                # 补全1个字节
                padding_bits = 8 - (total_bits % 8)
                total_bytes = math.ceil(total_bits / 8)
                # 写记录字节大小
                f.write(total_bytes.to_bytes(1))
                # 写地区码和类型
                code_int = int(code) << 4
                code_type = code_int + region_type.value
                f.write(code_type.to_bytes(3))
                # 写字符和废止年份，
                last_int = 0
                last_bits = 0
                u8_list: list[int] = []
                # 一个name_int占据12bit， 先分成4bit，再2组组合成8位整数
                for i, c in enumerate(name_char_index_list):
                    v = 0
                    # 不是第一个
                    if i:
                        # last_int 占据高last_bits
                        if last_bits:
                            v += last_int << (8 - last_bits)
                            # 低(8-last_bits)位
                            v += c >> (4 + last_bits)
                            # 剩余8-last_bits位
                            last_int = c & (2 ** (4 + last_bits) - 1)
                            last_bits = (4 + last_bits) % 8
                            u8_list.append(v)
                            # 最后剩余0位，不能补给下一个数
                            if not last_bits and i == len(name_char_index_list) - 1:
                                u8_list.append(last_int)
                        else:
                            u8_list.append(last_int)
                            v = c >> 4
                            last_int = c & 0xF
                            last_bits = 4
                            u8_list.append(v)
                    else:
                        # 只取高8位
                        v = c >> 4
                        last_int = c & 0xF
                        last_bits = 4
                        u8_list.append(v)
                if last_bits:
                    if padding_bits:
                        if discard_year_int:
                            # 肯定是2个字节
                            u8_list.append(last_int << (8 - last_bits))
                            u8_list.append(discard_year_int)
                        else:
                            # 1个字节
                            u8_list.append(last_int << (8 - last_bits))
                    else:
                        if discard_year_int:
                            # 1个字节， 不会到这里
                            assert last_bits == 9999
                            u8_list.append((last_int << (8 - last_bits)) + (discard_year_int >> last_bits))
                        else:
                            u8_list.append(last_int << (8 - last_bits))
                else:
                    if discard_year_int:
                        u8_list.append(discard_year_int)
                for u8 in u8_list:
                    f.write(u8.to_bytes())
                index_offset += total_bytes
            # 写索引区偏移， 索引区偏移2个字节
            f.seek(4)
            f.write(index_offset.to_bytes(2))
            f.seek(index_offset)
            # 写索引区
            offset_codes = list(offset_map.keys())
            offset_codes.sort()
            for offset_code in offset_codes:
                # offset_code 高7位， offset 低17位，刚好3个字节
                combine = offset_code << 17
                combine += offset_map[offset_code]
                f.write(combine.to_bytes(3))
            # 写字符
            for char in char_list:
                f.write(char.encode('gbk'))
        return True

    def decode_u8_list(self, u8_list: list[int]) -> tuple[list[int], int]:
        # 按4bit分割， 再3个组合成12位
        four_bits = []
        for u8 in u8_list:
            four_bits.append((u8 & 0xF0) >> 4)
            four_bits.append(u8 & 0x0F)
        res = []
        for i in range(0, len(four_bits) // 3):
            a, b, c = four_bits[3 * i], four_bits[3 * i + 1], four_bits[3 * i + 2]
            num = (a << 8) + (b << 4) + c
            if num >= 64:
                res.append(num)
        discard_year_int = 0
        last_four_bit = four_bits[len(res) * 3 :]
        for i, b in enumerate(last_four_bit):
            discard_year_int += b << (4 * (len(last_four_bit) - i - 1))
        return res, discard_year_int

    def search(self, region_code: str) -> tuple[str, list[str]]:
        if len(region_code) != 6:
            raise ValueError('地区编码必须为6位')
        with open(self.file_name, 'rb') as f:
            # version = int.from_bytes(f.read(4), byteorder="big")
            # 跳过版本号
            f.seek(4)
            index_offset = int.from_bytes(f.read(2), byteorder='big')
            # 查找 索引区
            code_2 = int(region_code[:2])
            f.seek(index_offset)
            offset = 0
            for _ in range(0, 34):
                combine_bytes = f.read(3)
                if not combine_bytes:
                    return '', []
                combine = int.from_bytes(combine_bytes)
                code = combine >> 17
                if code_2 == code:
                    offset = combine % (2**17)
                    break
            if not offset:
                return '', []
            # 找到字符区
            f.seek(index_offset + 34 * 3)
            char_bytes = f.read()
            chars = char_bytes.decode('gbk')
            char_map = {}
            for i, c in enumerate(chars):
                char_map[64 + i] = c
            # 找到记录区
            f.seek(offset)
            province_record = f.read(4000)
            # 一级行政区， 二级行政区, 查询的区号
            search_codes = [
                f'{region_code[:2]}0000',
                f'{region_code[:4]}00',
                region_code,
            ]
            result = []
            while province_record:
                # 前1字节是size
                size = province_record[0]
                record = province_record[:size]
                # 地区码 + 类型, 高20位是区号，低4位是类型
                code_type = int.from_bytes(province_record[1:4])
                code_int = code_type >> 4
                # 已经到别的省份
                if str(code_int)[:2] != region_code[:2]:
                    return '', result
                if str(code_int) in search_codes:
                    record_1 = record[4:]
                    u8_list = list(record_1)
                    decode_list, discard_year_int = self.decode_u8_list(u8_list)
                    if discard_year_int:
                        discard_year_int += 1980
                    names = []
                    for index in decode_list:
                        names.append(char_map[index])
                    name = ''.join(names)
                    # 类型
                    region_type = code_type % code_int
                    if not discard_year_int:
                        result.append(f'{name}{RegionType(region_type).label}')
                    else:
                        result.append(f'{name}{RegionType(region_type).label} ({discard_year_int}年废止)')
                    if str(code_int) == region_code:
                        return ''.join(result), result
                province_record = province_record[size:]
        return '', []
