#include "alloywindow.h"
#include "alloyplatform.h"

extern "C" {
#include "wayland_client.h"
}

QT_BEGIN_NAMESPACE

QAlloyWindow::QAlloyWindow(QWindow *window, unsigned int surfaceId)
    : QPlatformWindow(window)
    , m_surfaceId(surfaceId)
{
    setGeometry(QRect(0, 0, 800, 600));
}

QAlloyWindow::~QAlloyWindow()
{
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (integration)
        integration->unregisterSurface(m_surfaceId);
}

void QAlloyWindow::setGeometry(const QRect &rect)
{
    QPlatformWindow::setGeometry(rect);

    // Send position to compositor via Alloy-specific protocol
    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (integration) {
        struct wl_display *d = static_cast<struct wl_display *>(integration->display());
        if (d) {
            wl_surface_set_position(d->fd, m_surfaceId, rect.x(), rect.y());

            // Heuristic: if window is 48px tall and at bottom, it's a panel (z=1)
            // Otherwise it's a background window (z=0)
            unsigned int z = (rect.height() == 48) ? 1 : 0;
            wl_surface_set_zorder(d->fd, m_surfaceId, z);
        }
    }
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
