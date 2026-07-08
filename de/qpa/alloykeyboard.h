#ifndef ALLOYKEYBOARD_H
#define ALLOYKEYBOARD_H

#include <QtCore/QObject>
#include <QtCore/Qt>

QT_BEGIN_NAMESPACE

class QAlloyKeyboard : public QObject
{
    Q_OBJECT

public:
    explicit QAlloyKeyboard(QObject *parent = nullptr);

    void handleKey(int evdevKey, int state);
    void handleEnter(int surfaceId);
    void handleLeave(int surfaceId);
    void handleModifiers(int depressed, int latched, int locked, int group);

    static int translateKeyCode(int evdevKey);
    static QString keyText(int qtKey, Qt::KeyboardModifiers mods);

    Qt::KeyboardModifiers modifiers() const { return m_modifiers; }
    int focusedSurface() const { return m_focusedSurface; }

private:
    void updateModifiers(int evdevKey, bool pressed);

    Qt::KeyboardModifiers m_modifiers;
    int m_focusedSurface;
};

QT_END_NAMESPACE

#endif
