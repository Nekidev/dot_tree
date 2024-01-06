from dataclasses import dataclass

from bitstring import ConstBitStream

from exceptions import InvalidFileIdentifier


@dataclass
class SubItem:
    length: int


class Tree:
    subitems: list

    @staticmethod
    def open(file_path) -> "Tree":
        with ConstBitStream(filename=file_path) as stream:
            identifier = stream.read(8 * 8)
            if identifier.bytes != b"NEKOTREE":
                raise InvalidFileIdentifier(
                    f"Invalid file identifier: {identifier.bytes.decode('utf-8')}"
                )

    @staticmethod
    def create(file_path: str, subitems: list[SubItem]) -> "Tree":
        pass
