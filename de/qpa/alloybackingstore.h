#ifndef ALLOYBACKINGSTORE_H
#define ALLOYBACKINGSTORE_H

#include <QtGui/qpa/qplatformbackingstore.h>
#include <QtGui/QImage>

QT_BEGIN_NAMESPACE

class QAlloyBackingStore : public QPlatformBackingStore
{
public:
    QAlloyBackingStore(QWindow *window);
    ~QAlloyBackingStore();

    QPaintDevice *paintDevice() override;
    void flush(QWindow *window, const QRegion &region, const QPoint &offset) override;
    void resize(const QSize &size, const QRegion &staticContents) override;
    bool scroll(const QRegion &area, int dx, int dy) override;
    void beginPaint(const QRegion &region) override;
    void endPaint() override;

private:
    void allocateShmBuffer(const QSize &size);
    void releaseShmBuffer();

    QImage m_image;
    int m_shmFd;
    void *m_shmPtr;
    QSize m_bufferSize;
    QRegion m_dirtyRegion;
};

QT_END_NAMESPACE

#endif
