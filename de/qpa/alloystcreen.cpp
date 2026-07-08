#include "alloystcreen.h"
#include "alloyplatform.h"
#include "alloycursor.h"

QT_BEGIN_NAMESPACE

QAlloyScreen::QAlloyScreen()
    : m_geometry(0, 0, 1024, 768)
    , m_depth(32)
    , m_format(QImage::Format_ARGB32_Premultiplied)
    , m_physicalSize(271, 203) // ~96 DPI at 1024×768
{
}

QRect QAlloyScreen::geometry() const
{
    return m_geometry;
}

int QAlloyScreen::depth() const
{
    return m_depth;
}

QImage::Format QAlloyScreen::format() const
{
    return m_format;
}

QSizeF QAlloyScreen::physicalSize() const
{
    return m_physicalSize;
}

QPlatformCursor *QAlloyScreen::cursor() const
{
    return QAlloyIntegration::instance()->createPlatformCursor();
}

QT_END_NAMESPACE
