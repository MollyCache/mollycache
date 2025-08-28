#[derive(Debug, PartialEq)]
pub enum TokenTypes {
    // Keywords
    Create, Select, Insert, Table, From, Into, Values, Where,
    Update, Delete, Drop, Index,
    // Data Types 
    Integer, Real, Text, Blob, Null,
    // Constraints
    Primary, Key, Not, Unique, Default, AutoIncrement,
    // Clauses
    Order, By, Group, Having, Distinct, All, As,
    Inner, Left, Right, Full, Outer, Join, On, Union,
    Limit, Offset,
    // Logical Operators
    And, Or, In, Exists,
    Case, When, Then, Else, End,    
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
