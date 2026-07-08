#ifndef ALLOYSCREEN_H
#define ALLOYSCREEN_H

#include <QtGui/qpa/qplatformscreen.h>

QT_BEGIN_NAMESPACE

class QAlloyCursor;

class QAlloyScreen : public QPlatformScreen
{
public:
    QAlloyScreen();

    QRect geometry() const override;
    int depth() const override;
    QImage::Format format() const override;
    QSizeF physicalSize() const override;
    QPlatformCursor *cursor() const override;

private:
    QRect m_geometry;
    int m_depth;
    QImage::Format m_format;
    QSizeF m_physicalSize;
};

QT_END_NAMESPACE

#endif
