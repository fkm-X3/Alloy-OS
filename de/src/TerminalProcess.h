#ifndef TERMINALPROCESS_H
#define TERMINALPROCESS_H

#include <QObject>
#include <QProcess>

class TerminalProcess : public QObject
{
    Q_OBJECT
    Q_PROPERTY(bool running READ running NOTIFY runningChanged)

public:
    explicit TerminalProcess(QObject *parent = nullptr);
    ~TerminalProcess();

    bool running() const;

public slots:
    void start();
    void stop();
    void write(const QString &text);

signals:
    void outputReceived(const QString &text);
    void runningChanged();

private slots:
    void onReadyReadStdout();
    void onReadyReadStderr();
    void onProcessFinished(int exitCode, QProcess::ExitStatus status);

private:
    QProcess *m_process;
};

#endif
