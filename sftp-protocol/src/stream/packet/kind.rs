#[derive(Clone, Copy, Debug, Nom, Serialize_repr)]
#[repr(u8)]
pub enum PacketType {
	Init = 1,
	Version = 2,
	Open = 3,
	Close = 4,
	Read = 5,
	Write = 6,
	Lstat = 7,
	Fstat = 8,
	SetStat = 9,
	FSetStat = 10,
	OpenDir = 11,
	ReadDir = 12,
	Remove = 13,
	MkDir = 14,
	RmDir = 15,
	RealPath = 16,
	Stat = 17,
	Rename = 18,
	ReadLink = 19,
	Symlink = 20,
	Status = 101,
	Handle = 102,
	Data = 103,
	Name = 104,
	Attrs = 105,
	Extended = 200,
	ExtendedReply = 201,
	UNIMPLEMENTED = 255
}

