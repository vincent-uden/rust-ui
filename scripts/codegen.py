from abc import ABC, abstractmethod


class RustItem(ABC):
    @abstractmethod
    def to_string(self) -> str: ...


class RustModule(RustItem):
    def __init__(self, members: list[RustItem]):
        self.members = members

    def to_string(self):
        output = ""
        for member in self.members:
            output += "\n"
            output += member.to_string()
        output += ""
        return output


class RustSimpleEnum(RustItem):
    def __init__(self, name: str, variants: list[str], derives: list[str] = []):
        self.name = name
        self.variants = variants
        self.derives = derives

    def to_string(self) -> str:
        output = "#[derive("
        for derive in self.derives:
            output += derive
            output += ", "
        output = output[:-2:]
        output += ")]\n"

        output += "pub enum " + self.name + " {\n"
        for var in self.variants:
            output += f"    {var},\n"
        output += "}"

        return output
