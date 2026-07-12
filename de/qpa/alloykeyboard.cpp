#include "alloykeyboard.h"
#include "alloyplatform.h"
#include "alloywindow.h"

#include <QtGui/qpa/qwindowsysteminterface.h>
#include <QtGui/QWindow>

#include <cstdint>

QT_BEGIN_NAMESPACE

static const int KeyTbl[256] = {
    0,                    // 0  KEY_RESERVED
    Qt::Key_Escape,       // 1  KEY_ESC
    Qt::Key_1,            // 2  KEY_1
    Qt::Key_2,            // 3  KEY_2
    Qt::Key_3,            // 4  KEY_3
    Qt::Key_4,            // 5  KEY_4
    Qt::Key_5,            // 6  KEY_5
    Qt::Key_6,            // 7  KEY_6
    Qt::Key_7,            // 8  KEY_7
    Qt::Key_8,            // 9  KEY_8
    Qt::Key_9,            // 10 KEY_9
    Qt::Key_0,            // 11 KEY_0
    Qt::Key_Minus,        // 12 KEY_MINUS
    Qt::Key_Equal,        // 13 KEY_EQUAL
    Qt::Key_Backspace,    // 14 KEY_BACKSPACE
    Qt::Key_Tab,          // 15 KEY_TAB
    Qt::Key_Q,            // 16 KEY_Q
    Qt::Key_W,            // 17 KEY_W
    Qt::Key_E,            // 18 KEY_E
    Qt::Key_R,            // 19 KEY_R
    Qt::Key_T,            // 20 KEY_T
    Qt::Key_Y,            // 21 KEY_Y
    Qt::Key_U,            // 22 KEY_U
    Qt::Key_I,            // 23 KEY_I
    Qt::Key_O,            // 24 KEY_O
    Qt::Key_P,            // 25 KEY_P
    Qt::Key_BracketLeft,  // 26 KEY_LEFTBRACE
    Qt::Key_BracketRight, // 27 KEY_RIGHTBRACE
    Qt::Key_Return,       // 28 KEY_ENTER
    Qt::Key_Control,      // 29 KEY_LEFTCTRL
    Qt::Key_A,            // 30 KEY_A
    Qt::Key_S,            // 31 KEY_S
    Qt::Key_D,            // 32 KEY_D
    Qt::Key_F,            // 33 KEY_F
    Qt::Key_G,            // 34 KEY_G
    Qt::Key_H,            // 35 KEY_H
    Qt::Key_J,            // 36 KEY_J
    Qt::Key_K,            // 37 KEY_K
    Qt::Key_L,            // 38 KEY_L
    Qt::Key_Semicolon,    // 39 KEY_SEMICOLON
    Qt::Key_Apostrophe,   // 40 KEY_APOSTROPHE
    Qt::Key_QuoteLeft,    // 41 KEY_GRAVE
    Qt::Key_Shift,        // 42 KEY_LEFTSHIFT
    Qt::Key_Backslash,    // 43 KEY_BACKSLASH
    Qt::Key_Z,            // 44 KEY_Z
    Qt::Key_X,            // 45 KEY_X
    Qt::Key_C,            // 46 KEY_C
    Qt::Key_V,            // 47 KEY_V
    Qt::Key_B,            // 48 KEY_B
    Qt::Key_N,            // 49 KEY_N
    Qt::Key_M,            // 50 KEY_M
    Qt::Key_Comma,        // 51 KEY_COMMA
    Qt::Key_Period,       // 52 KEY_DOT
    Qt::Key_Slash,        // 53 KEY_SLASH
    Qt::Key_Shift,        // 54 KEY_RIGHTSHIFT
    Qt::Key_Asterisk,     // 55 KEY_KPASTERISK (numpad)
    Qt::Key_Alt,          // 56 KEY_LEFTALT
    Qt::Key_Space,        // 57 KEY_SPACE
    Qt::Key_CapsLock,     // 58 KEY_CAPSLOCK
    Qt::Key_F1,           // 59 KEY_F1
    Qt::Key_F2,           // 60 KEY_F2
    Qt::Key_F3,           // 61 KEY_F3
    Qt::Key_F4,           // 62 KEY_F4
    Qt::Key_F5,           // 63 KEY_F5
    Qt::Key_F6,           // 64 KEY_F6
    Qt::Key_F7,           // 65 KEY_F7
    Qt::Key_F8,           // 66 KEY_F8
    Qt::Key_F9,           // 67 KEY_F9
    Qt::Key_F10,          // 68 KEY_F10
    Qt::Key_NumLock,      // 69 KEY_NUMLOCK
    Qt::Key_ScrollLock,   // 70 KEY_SCROLLLOCK
    Qt::Key_7,            // 71 KEY_KP7 (numpad)
    Qt::Key_8,            // 72 KEY_KP8
    Qt::Key_9,            // 73 KEY_KP9
    Qt::Key_Minus,        // 74 KEY_KPMINUS
    Qt::Key_4,            // 75 KEY_KP4
    Qt::Key_5,            // 76 KEY_KP5
    Qt::Key_6,            // 77 KEY_KP6
    Qt::Key_Plus,         // 78 KEY_KPPLUS
    Qt::Key_1,            // 79 KEY_KP1
    Qt::Key_2,            // 80 KEY_KP2
    Qt::Key_3,            // 81 KEY_KP3
    Qt::Key_0,            // 82 KEY_KP0
    Qt::Key_Period,       // 83 KEY_KPDOT
    0, 0,                 // 84-85
    Qt::Key_Backslash,    // 86 KEY_102ND (ISO key)
    Qt::Key_F11,          // 87 KEY_F11
    Qt::Key_F12,          // 88 KEY_F12
    0, 0, 0, 0, 0, 0, 0,  // 89-95
    Qt::Key_Enter,        // 96 KEY_KPENTER (numpad enter)
    Qt::Key_Control,      // 97 KEY_RIGHTCTRL
    Qt::Key_Slash,        // 98 KEY_KPSLASH (numpad)
    Qt::Key_SysReq,       // 99 KEY_SYSRQ
    Qt::Key_Alt,          // 100 KEY_RIGHTALT
    0,                    // 101 KEY_LINEFEED (removed in Qt6)
    Qt::Key_Home,         // 102 KEY_HOME
    Qt::Key_Up,           // 103 KEY_UP
    Qt::Key_PageUp,       // 104 KEY_PAGEUP
    Qt::Key_Left,         // 105 KEY_LEFT
    Qt::Key_Right,        // 106 KEY_RIGHT
    Qt::Key_End,          // 107 KEY_END
    Qt::Key_Down,         // 108 KEY_DOWN
    Qt::Key_PageDown,     // 109 KEY_PAGEDOWN
    Qt::Key_Insert,       // 110 KEY_INSERT
    Qt::Key_Delete,       // 111 KEY_DELETE
    0,                    // 112 KEY_MACRO
    0, 0, 0,              // 113-115 volume keys
    Qt::Key_PowerOff,     // 116 KEY_POWER
    0, 0,                 // 117-118
    Qt::Key_Pause,        // 119 KEY_PAUSE
    0,                    // 120 KEY_SCALE
    0, 0, 0, 0,           // 121-124
    Qt::Key_Meta,         // 125 KEY_LEFTMETA
    Qt::Key_Meta,         // 126 KEY_RIGHTMETA
};

static const int ShiftMap[256] = {
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    '!', '@', '#', '$', '%', '^', '&', '*', '(', ')',
    '_', '+', 0, 0, 0, 0,
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P',
    '{', '}', '|', 0, 0, 0,
    'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':',
    '"', 0, 0, 0, 0, 0,
    'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?',
};

QAlloyKeyboard::QAlloyKeyboard(QObject *parent)
    : QObject(parent)
    , m_modifiers(Qt::NoModifier)
    , m_focusedSurface(-1)
{
}

void QAlloyKeyboard::handleKey(int evdevKey, int state)
{
    int qtKey = translateKeyCode(evdevKey);
    if (qtKey <= 0)
        return;

    bool pressed = (state != 0);
    updateModifiers(evdevKey, pressed);

    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (!integration)
        return;

    QWindow *win = nullptr;
    if (m_focusedSurface >= 0)
        win = integration->windowForSurface(
            static_cast<unsigned int>(m_focusedSurface));

    QString text;
    if (pressed && qtKey >= 0x20 && qtKey <= 0x7e)
        text = keyText(qtKey, m_modifiers);
    else if (pressed && qtKey == Qt::Key_Return)
        text = QStringLiteral("\n");
    else if (pressed && qtKey == Qt::Key_Tab)
        text = QStringLiteral("\t");
    else if (pressed && qtKey == Qt::Key_Space)
        text = QStringLiteral(" ");
    else if (pressed && qtKey == Qt::Key_Escape)
        text = QStringLiteral("\x1b");
    else if (pressed && qtKey == Qt::Key_Backspace)
        text = QStringLiteral("\b");

    ulong time = 0;
    QEvent::Type type = pressed ? QEvent::KeyPress : QEvent::KeyRelease;

    QWindowSystemInterface::handleKeyEvent(win, time, type, qtKey, m_modifiers, text);
}

void QAlloyKeyboard::handleEnter(int surfaceId)
{
    m_focusedSurface = surfaceId;
}

void QAlloyKeyboard::handleLeave(int surfaceId)
{
    if (m_focusedSurface == surfaceId)
        m_focusedSurface = -1;
}

void QAlloyKeyboard::handleModifiers(int depressed, int, int, int)
{
    Qt::KeyboardModifiers mods = Qt::NoModifier;
    if (depressed & (1 << 0)) mods |= Qt::ShiftModifier;
    if (depressed & (1 << 2)) mods |= Qt::ControlModifier;
    if (depressed & (1 << 3)) mods |= Qt::AltModifier;
    if (depressed & (1 << 4)) mods |= Qt::MetaModifier;
    m_modifiers = mods;
}

int QAlloyKeyboard::translateKeyCode(int evdevKey)
{
    if (evdevKey >= 0 && evdevKey < 256)
        return KeyTbl[evdevKey];
    return 0;
}

QString QAlloyKeyboard::keyText(int qtKey, Qt::KeyboardModifiers mods)
{
    Q_UNUSED(mods);
    if (qtKey >= 0x20 && qtKey <= 0x7e) {
        int c = qtKey;
        if ((mods & Qt::ShiftModifier) && c >= 'a' && c <= 'z')
            c = c - 'a' + 'A';
        if (c >= 0 && c < 256 && ShiftMap[c])
            c = ShiftMap[c];
        return QString(QChar(static_cast<ushort>(c)));
    }
    return QString();
}

void QAlloyKeyboard::updateModifiers(int evdevKey, bool pressed)
{
    int bit = 0;
    switch (evdevKey) {
    case 42: case 54: bit = Qt::ShiftModifier; break;  // LSHIFT, RSHIFT
    case 29: case 97: bit = Qt::ControlModifier; break; // LCTRL, RCTRL
    case 56: case 100: bit = Qt::AltModifier; break;    // LALT, RALT
    case 125: case 126: bit = Qt::MetaModifier; break;  // LMETA, RMETA
    default: return;
    }
    if (pressed)
        m_modifiers |= static_cast<Qt::KeyboardModifier>(bit);
    else
        m_modifiers &= ~static_cast<Qt::KeyboardModifier>(bit);
}

QT_END_NAMESPACE

#include "alloykeyboard.moc"
