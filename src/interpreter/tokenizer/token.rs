#[derive(Debug, PartialEq)]
pub enum TokenTypes {
    // Keywords
    Create, Select, Insert, Table, From, Into, Values, Where,
    Update, Delete, Add, Drop, Index, Set, Alter, Rename, 
    To, Column,
    // Data Types 
    Integer, Real, Text, Blob, Null,
    // Constraints
    Primary, Key, Not, Unique, Default, AutoIncrement,
    // Clauses
    Order, By, Group, Having, Distinct, All, As, Asc, Desc,
    Inner, Left, Right, Full, Outer, Join, On,
    Limit, Offset, Union, Intersect, Except,
    // Logical Operators
    And, Or, In, Exists, If,
    Case, When, Then, Else, End, Is,
    Equals, NotEquals, LessThan, LessEquals, GreaterThan, GreaterEquals,
    // Aggregate Functions
    Count, Sum, Avg, Min, Max,
    // Single Character Tokens
    Asterisk, SemiColon, LeftParen, RightParen, Comma, Dot,
    // Math Operators
    Plus, Minus, Divide, Modulo,       
    // Literals
    String, IntLiteral, True, False, HexLiteral, RealLiteral,
    // Others
    Identifier,
    EOF, Error,
}
