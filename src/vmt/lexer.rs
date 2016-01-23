use std::fmt;
use std::str::Chars;
use vmt::error::{VMTLoadResult, VMTLoadError, VMTError};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token<'s> {
    /// Start of file
    Start,
    BlockStart,
    BlockEnd,
    ShaderType(&'s str),
    /// Shader parameter type
    ParamType(&'s str),
    /// Shader parameter value
    ParamValue(&'s str),
    /// 'Proxies' statement
    ProxyDecl,
    ProxyType(&'s str),
    /// End of file
    End
}

impl<'s> fmt::Display for Token<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Token::ShaderType(ref s)   => write!(f, "{}", s),
            &Token::ParamType(ref s)   => write!(f, "{}", s),
            &Token::ParamValue(ref s)   => write!(f, "{}", s),
            &Token::ProxyType(ref s)    => write!(f, "{}", s),
            other => write!(f, "{}", match other {
                &Token::BlockStart  => "{",
                &Token::BlockEnd    => "}",
                &Token::ProxyDecl   => "Proxies",
                _                   => unreachable!()
            })
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    /// End of file
    EoF,
    /// Triggered on a newline character
    Newline,
    /// Triggered on first `"`
    QuoteStart,
    /// Triggered on any character between two quotes
    QuoteChar,
    /// Triggered on second `"`
    QuoteEnd,
    /// Triggered on '{'
    BlockStart,
    /// Triggered on '}'
    BlockEnd,
    /// Triggered on '<'
    ConAboveStarted,
    /// Triggered on '>'
    ConBelowStarted,
    /// Triggered on '<='
    ConAboveEqual,
    /// Triggered on '>='
    ConBelowEqual,
    /// Triggered on any whitespace character that isn't between quotes
    Whitespace,
    /// Triggered on '/'. Becomes a char if it is not followed by a second '/'
    CommentStart,
    /// Triggered on '//'
    Comment,
    /// Any character that happens after //
    CommentChar,
    /// Triggered by any character that does not meet the above requirements.
    Char,
    /// The default state
    Default
}

pub struct Lexer<'s> {

    char_cursor: usize,
    source_str: &'s str,
    char_iter: Chars<'s>,
    last_state: State,
    state: State,
    pub tokens: Vec<Token<'s>>
}

impl<'s> Lexer<'s> {

    pub fn new(source_str: &'s str) -> VMTLoadResult<Lexer> {
        let mut token_vec = Vec::with_capacity(16);
        token_vec.push(Token::Start);

        let mut lexer = Lexer {
            char_cursor: 0,
            source_str: source_str,
            char_iter: source_str.chars(),
            last_state: State::Default,
            state: State::Default,
            tokens: token_vec
        };

        while try!(lexer.load_token()) != None {}

        Ok(lexer)
    }

    /// Reads at least one token from the source string and pushes it/them
    /// to the token vector. Loads multiple tokens if the first token is
    /// bordered by a block start or block end without whitespace inbetween.
    ///
    /// Returns a result with the number of tokens read. If it returns None,
    /// the EoF has been reached. 
    fn load_token(&mut self) -> VMTLoadResult<Option<usize>> {
        if self.state == State::EoF {
            return Ok(None);
        }

        let mut str_pos = self.char_cursor;
        let mut str_len = 0;

        // To what degree you are loaded with money
        // (is actually how many tokens have been loaded)
        let mut loaded: usize = 0;

        loop {
            // Move the current state to the previous state
            self.last_state = self.state;

            if self.char_cursor >= self.source_str.len() {
                self.state = State::EoF;
                return Ok(None);
            }

            let chara = self.char_iter.next().unwrap();

            // Figure out state based on loaded character
            self.state = match chara {
                '\n'    => State::Newline,
                '"'    => match self.last_state {
                                State::QuoteChar |
                                State::QuoteStart => State::QuoteEnd,
                                _                   => State::QuoteStart,

                },

                '{'     => State::BlockStart,
                '}'     => State::BlockEnd,
                '<'     => State::ConAboveStarted,
                '>'     => State::ConBelowStarted,
                '='     => match self.last_state {
                                State::ConAboveStarted  => State::ConAboveEqual,
                                State::ConBelowStarted  => State::ConBelowEqual,
                                _                       => return Err(VMTLoadError::VMT(VMTError::InvalidToken)) // TODO: Add additional description to InvalidToken
                },

                ' '|'\t'|'\r'=> match self.last_state {
                                State::QuoteChar |
                                State::QuoteStart       => State::QuoteChar,
                                _                       => State::Whitespace
                },

                '/'     => match self.last_state {
                                State::CommentStart     => State::Comment,
                                State::QuoteStart |
                                State::QuoteChar        => State::QuoteChar,
                                _                       => State::CommentStart
                },

                _       => match self.last_state {
                                State::QuoteChar |
                                State::QuoteStart       => State::QuoteChar,
                                _                       => State::Char
                }
            };

            // If the lexer has loaded the start of a quote, move the 
            // start of the slice past the quote. Also skips past any
            // whitespace. Otherwise, increase the length of the string 
            // slice.
            match self.state {
                State::Newline |
                State::QuoteStart   => str_pos += 1,
                
                // This detects if the whitespace comes right after a character, which
                // would be the case if it were coming off of a non-quoted parameter.
                // If it detects that, then it shouldn't move the cursor forward and should
                // behave like a QuoteEnd.
                State::Whitespace   => match self.last_state {
                                            State::Char => (),
                                            _           => str_pos += 1
                },

                State::QuoteEnd     => (),
                _                   => str_len += 1
            }
            self.char_cursor += 1;

            //println!("{:?}\t{:?}\t{}", self.state, chara, str_len);

            if self.state == State::QuoteEnd ||
               self.state == State::BlockStart ||
               self.state == State::BlockEnd ||
               self.state == State::Whitespace {

                // Only pushes token if the lexer has loaded more than just whitespace
                // since the last 
                if str_len != 0 {
                    let last = self.tokens[self.tokens.len() - 1];

                    self.tokens.push(match last {
                                        Token::Start            => Token::ShaderType(&self.source_str[str_pos..str_len+str_pos]),
                                        Token::BlockStart |
                                        Token::ParamValue(_)    => Token::ParamType(&self.source_str[str_pos..str_len+str_pos]),
                                        Token::ParamType(_)     => Token::ParamValue(&self.source_str[str_pos..str_len+str_pos]),
                                        _                       => match self.state {
                                                                        State::BlockStart   => Token::BlockStart,
                                                                        State::BlockEnd     => Token::BlockEnd,
                                                                        _                   => return Err(VMTLoadError::VMT(VMTError::SyntaxError))
                                        }
                    });
                    loaded += 1;
                }

                match self.state {
                    State::BlockStart   => {self.tokens.push(Token::BlockStart);
                                            return Ok(Some(loaded + 1))},
                    State::BlockEnd     => {self.tokens.push(Token::BlockEnd);
                                            return Ok(Some(loaded + 1))},
                    _                   => return Ok(Some(loaded))
                }
            }
        }
    }
}