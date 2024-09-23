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
    other = 10  # 其他

    @property
    def label(self) -> str:
        if self == RegionType.province:
            return "省"
        elif self == RegionType.z_province:
            return "自治区"
        elif self == RegionType.city:
            return "市"
        elif self == RegionType.district:
            return "区"
        elif self == RegionType.county:
            return "县"
        elif self == RegionType.z_county:
            return "自治县"
        elif self == RegionType.qi:
            return "旗"
        elif self == RegionType.meng:
            return "盟"
        elif self == RegionType.z_city:
            return "州"
        return ""

    @staticmethod
    def name_classifiction(name: str) -> tuple[str, "RegionType"]:
        if name.endswith("自治区"):
            return name[:-3], RegionType.z_province
        elif name.endswith("自治县"):
            return name[:-3], RegionType.z_county
        elif name.endswith("省"):
            return name[:-1], RegionType.province
        elif name.endswith("市"):
            return name[:-1], RegionType.city
        elif name.endswith("县"):
            return name[:-1], RegionType.county
        elif name.endswith("区"):
            return name[:-1], RegionType.district
        elif name.endswith("盟"):
            return name[:-1], RegionType.meng
        elif name.endswith("州"):
            return name[:-1], RegionType.z_city
        elif name.endswith("旗"):
            return name[:-1], RegionType.qi
        return name, RegionType.other


# 记录分割字符
SPLIT_CHAR = "¦"


class RegionCtr:
    def __init__(self, file_name: str = "region.dat") -> None:
        self.file_name = file_name

    @staticmethod
    def region_code_to_bytes(code: str) -> tuple[int, int, int]:
        """region_code_to_bytes.
        将地区码转成3个字节

        :param code:
        :type code: str
        :rtype: tuple[int, int, int]
        """
        code_i = int(code)
        res = []
        for i in range(16, -1, -8):
            div = code_i // 2**i
            code_i = code_i % 2**i
            res.append(div)
        return tuple(res)

    @staticmethod
    def bytes_to_region_code(code_bytes: bytes) -> str:
        if len(code_bytes) != 3:
            raise ValueError("bytes's length must be 3")
        res = 0
        for i in range(2, -1, -1):
            res += code_bytes[2 - i] * 2 ** (8 * i)
        return str(res)

    def pack(self, data_list: list[tuple[str, str]], version: int = 202301) -> bool:
        with open(self.file_name, "wb") as f:
            # 写32位版本号
            f.write(version.to_bytes(length=4))
            # 先跳过偏移
            f.seek(8)
            # 写数据, code [i:3] type [i: 1] region [c: n] \n
            index_offset = 8
            offset_map: dict[int, int] = {}
            for data in data_list:
                code, name = data
                code_2 = int(code[:2])
                if code_2 not in offset_map:
                    offset_map[code_2] = index_offset
                code_bytes = self.region_code_to_bytes(code)
                name, t = RegionType.name_classifiction(name.replace("*", ""))
                # 写区号
                for code_byte in code_bytes:
                    f.write(code_byte.to_bytes(1))
                # 写类型
                f.write(t.value.to_bytes(1))
                # 写地区名
                name_bytes = name.encode()
                f.write(name_bytes)
                f.write(SPLIT_CHAR.encode())
                index_offset += 3 + 1 + len(name_bytes) + len(SPLIT_CHAR.encode())
            # 写索引区偏移
            f.seek(4)
            f.write(index_offset.to_bytes(4))
            f.seek(index_offset)
            # 写索引区
            offset_codes = list(offset_map.keys())
            offset_codes.sort()
            for offset_code in offset_codes:
                f.write(offset_code.to_bytes(1))
                f.write(offset_map[offset_code].to_bytes(4))
        return True

    def search(self, region_code: str) -> tuple[str, list[str]]:
        if len(region_code) != 6:
            raise ValueError("地区编码必须为6位")
        with open(self.file_name, "rb") as f:
            # version = int.from_bytes(f.read(4), byteorder="big")
            # 跳过版本号
            f.seek(4)
            index_offset = int.from_bytes(f.read(4), byteorder="big")
            # 查找 索引区
            code_2 = int(region_code[:2])
            f.seek(index_offset)
            offset = 0
            for _ in range(0, 50):
                index = f.read(5)
                if not index:
                    return "", []
                if code_2 == index[0]:
                    offset = int.from_bytes(index[1:])
                    break
            if not offset:
                return "", []
            # 找到记录区
            f.seek(offset)
            # ( 6 + 3 * 2) * 250 ，250个应该足够了
            province_record = f.read(3000)
            province_record_splits = province_record.split(SPLIT_CHAR.encode())
            # 一级行政区， 二级行政区, 查询的区号
            search_codes = [
                f"{region_code[:2]}0000",
                f"{region_code[:4]}00",
                region_code,
            ]
            result = []
            for line in province_record_splits:
                code_tmp = self.bytes_to_region_code(line[:3])
                # 已经到别的省份
                if code_tmp[:2] != region_code[:2]:
                    return "", []
                # 找到
                if code_tmp in search_codes:
                    t = line[3]
                    name = line[4:].decode()
                    result.append(f"{name}{RegionType(t).label}")
                    if code_tmp == region_code:
                        return "".join(result), result
        return "", []
