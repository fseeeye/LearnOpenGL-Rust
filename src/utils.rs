use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenGlError {
    /// 0x0500, Given when an enumeration parameter is not a legal enumeration for that function. This is given only for
    /// local problems; if the spec allows the enumeration in certain circumstances, where other parameters or state
    /// dictate those circumstances, then `GL_INVALID_OPERATION` is the result instead.
    #[error("(0x0500)Illegal enumeration parameter")]
    InvalidEnum = gl::INVALID_ENUM as isize,
    /// 0x0501, Given when a value parameter is not a legal value for that function. This is only given for local
    /// problems; if the spec allows the value in certain circumstances, where other parameters or state dictate those
    /// circumstances, then `GL_INVALID_OPERATION` is the result instead.
    #[error("(0x0501)Illegal value parameter")]
    InvalidValue = gl::INVALID_VALUE as isize,
    /// 0x0502, Given when the set of state for a command is not legal for the parameters given to that command. It is
    /// also given for commands where combinations of parameters define what the legal parameters are.
    #[error("(0x0502)The set of state for a command is illegal for the parameters")]
    InvalidOperation = gl::INVALID_OPERATION as isize,
    /// 0x0503, Given when a stack pushing operation cannot be done because it would overflow the limit of that stack's
    /// size.
    #[error("(0x0503)Stack overflow when stack do pushing operation")]
    StackOverflow = gl::STACK_OVERFLOW as isize,
    /// 0x0504, Given when a stack popping operation cannot be done because the stack is already at its lowest point.
    #[error("(0x0504)Stack underflow when stack do popping operation")]
    StackUnderflow = gl::STACK_UNDERFLOW as isize,
    /// 0x0505, Given when performing an operation that can allocate memory, and the memory cannot be allocated. The
    /// results of OpenGL functions that return this error are undefined; it is allowable for partial execution of an
    /// operation to happen in this circumstance.
    #[error("(0x0505)No memory could be allocated")]
    OutOfMemory = gl::OUT_OF_MEMORY as isize,
    /// 0x0506, Given when doing anything that would attempt to read from or write/render to a framebuffer that is not
    /// complete.
    #[error("(0x0506)Operate a framebuffer which is not complete")]
    InvalidFramebufferOperation = gl::INVALID_FRAMEBUFFER_OPERATION as isize,
    /// 0x0507 (with OpenGL 4.5 or ARB_KHR_robustness), Given if the OpenGL context has been lost, due to a graphics
    /// card reset.
    #[error("(0x0507)OpenGL context has been lost")]
    ContextLost = gl::CONTEXT_LOST as isize,
    // #[error("")]
    // TableTooLarge = gl::TABLE_TOO_LARGE as isize, // Part of the ARB_imaging extension.
}

pub fn clear_gl_error() {
    while unsafe { gl::GetError() } != gl::NO_ERROR {}
}

pub fn get_gl_error() -> Option<OpenGlError> {
    let err = unsafe { gl::GetError() };
    match err {
        gl::NO_ERROR => None,
        gl::INVALID_ENUM => Some(OpenGlError::InvalidEnum),
        gl::INVALID_VALUE => Some(OpenGlError::InvalidValue),
        gl::INVALID_OPERATION => Some(OpenGlError::InvalidOperation),
        gl::STACK_OVERFLOW => Some(OpenGlError::StackOverflow),
        gl::STACK_UNDERFLOW => Some(OpenGlError::StackUnderflow),
        gl::OUT_OF_MEMORY => Some(OpenGlError::OutOfMemory),
        gl::INVALID_FRAMEBUFFER_OPERATION => Some(OpenGlError::InvalidFramebufferOperation),
        _ => {
            panic!("Unknow GlError: {}", err);
        }
    }
}
