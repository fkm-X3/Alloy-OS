#include "alloymouse.h"
#include "alloyplatform.h"
#include "alloywindow.h"

#include <QtGui/qpa/qwindowsysteminterface.h>
#include <QtGui/QWindow>

QT_BEGIN_NAMESPACE

QAlloyMouse::QAlloyMouse(QObject *parent)
    : QObject(parent)
    , m_focusedSurface(-1)
    , m_pos(0, 0)
{
    m_buttons[0] = false;
    m_buttons[1] = false;
    m_buttons[2] = false;
}

static QWindow *surfaceWindow(int surfaceId)
{
    if (surfaceId < 0)
        return nullptr;
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (!integration)
        return nullptr;
    return integration->windowForSurface(static_cast<unsigned int>(surfaceId));
}

void QAlloyMouse::handleMotion(int x, int y)
{
    m_pos = QPoint(x, y);
    QWindow *win = surfaceWindow(m_focusedSurface);

    QWindowSystemInterface::handleMouseEvent(win, QPointF(x, y), QPointF(x, y),
                                             translateButtons(), Qt::NoButton,
                                             QEvent::MouseMove, Qt::NoModifier);
}

void QAlloyMouse::handleButton(int button, int state, int x, int y)
{
    m_pos = QPoint(x, y);

    int idx = -1;
    if (button == 0x110) idx = 0;
    else if (button == 0x111) idx = 1;
    else if (button == 0x112) idx = 2;

    bool pressed = (state != 0);
    if (idx >= 0 && idx < 3)
        m_buttons[idx] = pressed;

    QWindow *win = surfaceWindow(m_focusedSurface);

    QEvent::Type type = pressed ? QEvent::MouseButtonPress : QEvent::MouseButtonRelease;

    QWindowSystemInterface::handleMouseEvent(win, QPointF(x, y), QPointF(x, y),
                                             translateButtons(),
                                             translateButton(button),
                                             type);
}

void QAlloyMouse::handleAxis(int axis, int value)
{
    QWindow *win = surfaceWindow(m_focusedSurface);

    QPoint pixelDelta;
    QPoint angleDelta;
    if (axis == 0) {
        angleDelta = QPoint(0, value * 15);
        pixelDelta = QPoint(0, value * 10);
    } else {
        angleDelta = QPoint(value * 15, 0);
        pixelDelta = QPoint(value * 10, 0);
    }

    QWindowSystemInterface::handleWheelEvent(win, QPointF(m_pos), QPointF(m_pos),
                                             pixelDelta, angleDelta,
                                             Qt::NoModifier, Qt::ScrollUpdate);
}

void QAlloyMouse::handleEnter(int surfaceId, int x, int y)
{
    m_focusedSurface = surfaceId;
    m_pos = QPoint(x, y);

    QWindow *win = surfaceWindow(surfaceId);
    QWindowSystemInterface::handleEnterEvent(win, QPointF(x, y), QPointF(x, y));
}

void QAlloyMouse::handleLeave(int surfaceId)
{
    if (m_focusedSurface == surfaceId)
        m_focusedSurface = -1;

    QWindow *win = surfaceWindow(surfaceId);
    QWindowSystemInterface::handleLeaveEvent(win);
}

Qt::MouseButtons QAlloyMouse::translateButtons() const
{
    Qt::MouseButtons btns = Qt::NoButton;
    if (m_buttons[0]) btns |= Qt::LeftButton;
    if (m_buttons[1]) btns |= Qt::RightButton;
    if (m_buttons[2]) btns |= Qt::MiddleButton;
    return btns;
}

Qt::MouseButton QAlloyMouse::translateButton(int waylandButton) const
{
    switch (waylandButton) {
    case 0x110: return Qt::LeftButton;
    case 0x111: return Qt::RightButton;
    case 0x112: return Qt::MiddleButton;
    default: return Qt::NoButton;
    }
}

QT_END_NAMESPACE
