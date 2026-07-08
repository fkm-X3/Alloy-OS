#include "alloywindow.h"
#include "alloyplatform.h"

QT_BEGIN_NAMESPACE

QAlloyWindow::QAlloyWindow(QWindow *window, unsigned int surfaceId)
    : QPlatformWindow(window)
    , m_surfaceId(surfaceId)
{
    setGeometry(QRect(0, 0, 800, 600));
}

void QAlloyWindow::setGeometry(const QRect &rect)
{
    QPlatformWindow::setGeometry(rect);
}

void QAlloyWindow::setVisible(bool visible)
{
    QPlatformWindow::setVisible(visible);

    if (visible && !geometry().isNull()) {
        // Surface is already registered with the compositor.
        // Future: send xdg_shell set_minimized/set_maximized here.
    }
}

void QAlloyWindow::setWindowTitle(const QString &title)
{
    QPlatformWindow::setWindowTitle(title);
    // Future: xdg_toplevel_set_title when xdg_shell is supported
}

void QAlloyWindow::requestActivateWindow()
{
    // Future: compositor surface focus when protocol supports it
}

WId QAlloyWindow::winId() const
{
    return static_cast<WId>(m_surfaceId);
}

QT_END_NAMESPACE
