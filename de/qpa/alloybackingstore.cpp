#include "alloybackingstore.h"
#include "alloywindow.h"
#include "alloyplatform.h"

#include <QtGui/qpa/qplatformwindow.h>
#include <QtGui/QWindow>
#include <QtGui/QGuiApplication>
#include <QtGui/QScreen>

extern "C" {
#include "wayland_client.h"
}

#include <cstring>

QT_BEGIN_NAMESPACE

QAlloyBackingStore::QAlloyBackingStore(QWindow *window)
    : QPlatformBackingStore(window)
    , m_shmFd(-1)
    , m_shmPtr(nullptr)
{
}

QAlloyBackingStore::~QAlloyBackingStore()
{
    releaseShmBuffer();
}

QPaintDevice *QAlloyBackingStore::paintDevice()
{
    return &m_image;
}

void QAlloyBackingStore::beginPaint(const QRegion &region)
{
    // QImage wrapping SHM memory is already set up in resize()
    // The region indicates which area the QPainter will draw into.
    (void)region;
}

void QAlloyBackingStore::endPaint()
{
    // Accumulate dirty region for flush
}

void QAlloyBackingStore::flush(QWindow *window, const QRegion &region, const QPoint &offset)
{
    Q_UNUSED(offset);

    if (m_image.isNull())
        return;

    QAlloyIntegration *integration = QAlloyIntegration::instance();
    if (!integration)
        return;

    int fd = integration->displayFd();
    unsigned int compositor = integration->compositorId();

    // Find the surface for this window
    QAlloyWindow *platWin = static_cast<QAlloyWindow *>(window->handle());
    if (!platWin)
        return;

    unsigned int surfaceId = platWin->surfaceId();
    unsigned int bufferId = static_cast<unsigned int>(m_shmFd);

    // Attach the SHM buffer to the surface
    wl_surface_attach(fd, surfaceId, bufferId, 0, 0);

    // Mark the dirty region
    QRegion dirty = region.isEmpty() ? QRect(QPoint(0, 0), m_image.size()) : region;
    for (const QRect &r : dirty) {
        wl_surface_damage(fd, surfaceId, r.x(), r.y(), r.width(), r.height());
    }

    // Commit
    wl_surface_commit(fd, surfaceId);
}

void QAlloyBackingStore::resize(const QSize &size, const QRegion &staticContents)
{
    Q_UNUSED(staticContents);

    if (size == m_bufferSize && !m_image.isNull())
        return;

    releaseShmBuffer();
    allocateShmBuffer(size);
}

bool QAlloyBackingStore::scroll(const QRegion &area, int dx, int dy)
{
    if (m_image.isNull())
        return false;

    // Scroll within the SHM buffer by copying pixels
    QRect bounds = area.boundingRect();
    if (bounds.isEmpty())
        return false;

    int bpl = m_image.bytesPerLine();
    int pixelSize = m_image.depth() / 8;
    int rowBytes = bounds.width() * pixelSize;

    if (dy > 0 && dy < bounds.height()) {
        // Scroll down: copy from top to bottom
        for (int y = bounds.bottom() - dy; y >= bounds.top(); --y) {
            void *dst = m_image.scanLine(y + dy) + bounds.left() * pixelSize;
            const void *src = m_image.scanLine(y) + bounds.left() * pixelSize;
            std::memmove(dst, src, rowBytes);
        }
    } else if (dy < 0 && -dy < bounds.height()) {
        // Scroll up: copy from bottom to top
        for (int y = bounds.top() - dy; y <= bounds.bottom(); ++y) {
            void *dst = m_image.scanLine(y + dy) + bounds.left() * pixelSize;
            const void *src = m_image.scanLine(y) + bounds.left() * pixelSize;
            std::memmove(dst, src, rowBytes);
        }
    }

    if (dx != 0) {
        // Horizontal scroll: handle each row
        for (int y = bounds.top(); y <= bounds.bottom(); ++y) {
            void *line = m_image.scanLine(y) + bounds.left() * pixelSize;
            if (dx > 0 && dx < bounds.width()) {
                std::memmove(static_cast<char *>(line) + dx * pixelSize,
                             line,
                             (bounds.width() - dx) * pixelSize);
            } else if (dx < 0 && -dx < bounds.width()) {
                std::memmove(line,
                             static_cast<char *>(line) + (-dx) * pixelSize,
                             (bounds.width() + dx) * pixelSize);
            }
        }
    }

    m_dirtyRegion |= area;
    return true;
}

void QAlloyBackingStore::allocateShmBuffer(const QSize &size)
{
    m_bufferSize = size;

    if (size.isEmpty() || size.width() <= 0 || size.height() <= 0)
        return;

    QImage::Format fmt = QImage::Format_ARGB32_Premultiplied;
    if (QGuiApplication::primaryScreen() && QGuiApplication::primaryScreen()->handle())
        fmt = QGuiApplication::primaryScreen()->handle()->format();

    int bpp = 32;
    m_shmFd = alloy_shm_alloc(
        static_cast<unsigned int>(size.width()),
        static_cast<unsigned int>(size.height()),
        static_cast<unsigned int>(bpp));

    if (m_shmFd < 0) {
        // Fallback to heap-allocated QImage if SHM allocation fails
        m_image = QImage(size, fmt);
        m_shmPtr = nullptr;
        return;
    }

    m_shmPtr = alloy_shm_user_vaddr(m_shmFd);
    if (!m_shmPtr) {
        m_image = QImage(size, fmt);
        return;
    }

    int bytesPerLine = size.width() * (bpp / 8);
    m_image = QImage(static_cast<unsigned char *>(m_shmPtr),
                     size.width(), size.height(), bytesPerLine, fmt);
}

void QAlloyBackingStore::releaseShmBuffer()
{
    m_image = QImage();
    m_shmPtr = nullptr;
    // Note: SHM buffer close via syscall would go here.
    // The compositor may still reference the buffer, so we'd need
    // a proper release mechanism (wl_buffer.release callback).
    m_shmFd = -1;
    m_dirtyRegion = QRegion();
}

QT_END_NAMESPACE
