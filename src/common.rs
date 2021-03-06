use std::fmt;
use std::str::Chars;
use vmt::error::{VMTResult, VMTError};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Token<'s> {
    /// Start of file
    Start,
    BlockStart,
    BlockEnd,
    BlockType(&'s str),
    /// Shader parameter type
    ParamType(&'s str),
    /// Shader parameter value
    ParamValue(&'s str),
    /// End of file
    End
}

impl<'s> Token<'s> {
    pub fn get_inner_str(&self) -> Option<&'s str> {
        match self {
            &Token::BlockType(s)    => Some(s),
            &Token::ParamType(s)    => Some(s),
            &Token::ParamValue(s)   => Some(s),
            _               => None
        }
    }
}

impl<'s> fmt::Display for Token<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Token::BlockType(ref s)   => write!(f, "Block Type: {}", s),
            &Token::ParamType(ref s)   => write!(f, "Parameter Type: {}", s),
            &Token::ParamValue(ref s)   => write!(f, "Parameter Value: {}", s),
            other => write!(f, "{}", match other {
                &Token::Start       => "SoF",
                &Token::BlockStart  => "{",
                &Token::BlockEnd    => "}",
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

    pub fn new(source_str: &'s str) -> VMTResult<Lexer> {
        let mut token_vec = Vec::with_capacity(64);
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
        lexer.tokens.push(Token::End);

        Ok(lexer)
    }

    /// Reads at least one token from the source string and pushes it/them
    /// to the token vector. Loads multiple tokens if the first token is
    /// bordered by a block start or block end without whitespace inbetween.
    ///
    /// Returns a result with the number of tokens read. If it returns None,
    /// the EoF has been reached. 
    fn load_token(&mut self) -> VMTResult<Option<usize>> {
        if self.state == State::EoF {
            return Ok(None);
        }

        let mut str_pos = self.char_cursor;
        let mut str_len = 0;

        loop {
            // Move the current state to the previous state
            self.last_state = self.state;

            if self.char_cursor >= self.source_str.len() {
                self.state = State::EoF;
                return Ok(None);
            }

            let chara = self.char_iter.next().unwrap();

            // Figure out state based on loaded character. The only
            // character that can abort a comment is a newline, so this
            // accounts for that. Otherwise, any character that comes after
            // a comment MUST be a comment character and can be safely ignored
            // by the lexer
            if (self.last_state == State::Comment ||
                self.last_state == State::CommentChar) && chara != '\n' 
            {
                self.state = State::CommentChar;
            }
            else {
                self.state = match chara {
                    '\n'    => match self.last_state {
                                    State::QuoteChar |
                                    State::QuoteStart   => return Err(VMTError::SyntaxError("Unclosed quote".into())),
                                    _                   => State::Newline
                    },

                    '"'     => match self.last_state {
                                    State::QuoteChar |
                                    State::QuoteStart   => State::QuoteEnd,
                                    _                   => State::QuoteStart,
                    },

                    '{'     => State::BlockStart,
                    '}'     => State::BlockEnd,

                    ' ' |
                    '\t'|
                    '\r'    => match self.last_state {
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
            }
            

            // If you're coming from up top, read down below to see
            // why I exist. If you came from below, hi. How's life?
            // Are HMDs all they were hyped up to be? I don't know.
            // I'm just an if statement.
            if self.last_state == State::CommentStart && self.state != State::Comment {
                self.last_state = State::Char;
                str_len += 1;
            }

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

                // If the lexer has detected that a comment has been started, setting
                // the length of the string to zero will cause the lexer to skip it
                State::CommentChar  => str_len = 0,

                State::QuoteEnd |
                // With CommentStart, we aren't sure if it will become a comment or not.
                // If it does, we can safely ignore it - however, if it doesn't, we have
                // to increase str_len. We can't figure out which is going to happen until
                // we have the next state, so whether or not to increase the str_len is
                // deferred until then. Look up to see the if statement that checks that.
                State::CommentStart |
                State::Comment      => (),
                _                   => str_len += 1
            }
            self.char_cursor += 1;

            //println!("{:?}\t{:?}\t{}", self.state, chara, str_len);

            if self.state == State::QuoteEnd ||
               self.state == State::BlockStart ||
               self.state == State::BlockEnd ||
               self.state == State::Whitespace ||
               self.state == State::Comment {

                // Only pushes token if the lexer has loaded more than just whitespace
                // since the last 
                if str_len != 0 {
                    let last = self.tokens[self.tokens.len() - 1];
                    //println!("Last Token: {:?}", last);

                    match self.state {
                        // If the state is the start of a block, get the inner &str from the last token
                        // and place it in a new BlockType token. While redundant for the start of the file,
                        // this works well for inner blocks where whether a value is a BlockType or not is
                        // dependent on a character that comes after it.
                        State::BlockStart   => {let token = Token::BlockType(self.tokens.pop().unwrap().get_inner_str().unwrap());
                                                self.tokens.push(token)}
                        State::BlockEnd     => (),
                        _                   => match last {
                                                    Token::Start            => self.tokens.push(Token::BlockType(&self.source_str[str_pos..str_len+str_pos])),
                                                    Token::BlockStart |
                                                    Token::BlockEnd |
                                                    Token::ParamValue(_)    => self.tokens.push(Token::ParamType(&self.source_str[str_pos..str_len+str_pos])),
                                                    Token::ParamType(_)     => self.tokens.push(Token::ParamValue(&self.source_str[str_pos..str_len+str_pos])),
                                                    _                       => return Err(VMTError::SyntaxError("you dun goofed".into()))
                        }
                    }
                }

                match self.state {
                    State::BlockStart   => {
                        self.tokens.push(Token::BlockStart);
                        return Ok(Some(2));
                    }
                    State::BlockEnd     => {
                        self.tokens.push(Token::BlockEnd);
                        return Ok(Some(2));
                    }
                    _                   => return Ok(Some(1))
                }
            }
        }
    }
}