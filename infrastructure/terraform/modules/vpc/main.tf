resource "aws_vpc" "this" {
  cidr_block           = var.cidr_block
  enable_dns_support   = true
  enable_dns_hostnames = true

  tags = merge(
    {
      Name = "${var.name_prefix}-vpc"
    },
    var.tags
  )
}

resource "aws_internet_gateway" "this" {
  vpc_id = aws_vpc.this.id

  tags = merge(
    {
      Name = "${var.name_prefix}-igw"
    },
    var.tags
  )
}

resource "aws_subnet" "public" {
  count = length(var.public_subnet_cidr_blocks)

  vpc_id                  = aws_vpc.this.id
  cidr_block              = var.public_subnet_cidr_blocks[count.index]
  availability_zone       = var.azs[count.index]
  map_public_ip_on_launch = true

  tags = merge(
    {
      Name = "${var.name_prefix}-public-${count.index}"
      Type = "Public"
    },
    var.tags
  )
}

resource "aws_subnet" "private" {
  count = length(var.private_subnet_cidr_blocks)

  vpc_id            = aws_vpc.this.id
  cidr_block        = var.private_subnet_cidr_blocks[count.index]
  availability_zone = var.azs[count.index]

  tags = merge(
    {
      Name = "${var.name_prefix}-private-${count.index}"
      Type = "Private"
    },
    var.tags
  )
}

resource "aws_subnet" "isolated" {
  count = length(var.isolated_subnet_cidr_blocks)

  vpc_id            = aws_vpc.this.id
  cidr_block        = var.isolated_subnet_cidr_blocks[count.index]
  availability_zone = var.azs[count.index]

  tags = merge(
    {
      Name = "${var.name_prefix}-isolated-${count.index}"
      Type = "Isolated"
    },
    var.tags
  )
}

resource "aws_eip" "nat" {
  count = var.single_nat_gateway ? 1 : length(var.azs)

  domain = "vpc"

  tags = merge(
    {
      Name = "${var.name_prefix}-nat-eip-${count.index}"
    },
    var.tags
  )

  depends_on = [aws_internet_gateway.this]
}

resource "aws_nat_gateway" "this" {
  count         = var.single_nat_gateway ? 1 : length(var.azs)
  allocation_id = aws_eip.nat[count.index].id
  subnet_id     = aws_subnet.public[count.index].id

  tags = merge(
    {
      Name = "${var.name_prefix}-nat-${count.index}"
    },
    var.tags
  )

  depends_on = [aws_internet_gateway.this]
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.this.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.this.id
  }

  tags = merge(
    {
      Name = "${var.name_prefix}-public-rt"
    },
    var.tags
  )
}

resource "aws_route_table" "private" {
  count = var.single_nat_gateway ? 1 : length(var.azs)

  vpc_id = aws_vpc.this.id

  dynamic "route" {
    for_each = var.enable_nat_gateway ? [1] : []
    content {
      cidr_block     = "0.0.0.0/0"
      nat_gateway_id = aws_nat_gateway.this[count.index].id
    }
  }

  tags = merge(
    {
      Name = "${var.name_prefix}-private-rt-${count.index}"
    },
    var.tags
  )
}

resource "aws_route_table" "isolated" {
  vpc_id = aws_vpc.this.id

  tags = merge(
    {
      Name = "${var.name_prefix}-isolated-rt"
    },
    var.tags
  )
}

resource "aws_route_table_association" "public" {
  count = length(var.public_subnet_cidr_blocks)

  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

resource "aws_route_table_association" "private" {
  count = length(var.private_subnet_cidr_blocks)

  subnet_id      = aws_subnet.private[count.index].id
  route_table_id = var.single_nat_gateway ? aws_route_table.private[0].id : aws_route_table.private[count.index].id
}

resource "aws_route_table_association" "isolated" {
  count = length(var.isolated_subnet_cidr_blocks)

  subnet_id      = aws_subnet.isolated[count.index].id
  route_table_id = aws_route_table.isolated.id
}