#include "TerminalProcess.h"

TerminalProcess::TerminalProcess(QObject *parent)
    : QObject(parent)
    , m_process(new QProcess(this))
{
    connect(m_process, &QProcess::readyReadStandardOutput,
            this, &TerminalProcess::onReadyReadStdout);
    connect(m_process, &QProcess::readyReadStandardError,
            this, &TerminalProcess::onReadyReadStderr);
    connect(m_process,
            QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished),
            this, &TerminalProcess::onProcessFinished);
}

TerminalProcess::~TerminalProcess()
{
    stop();
}

bool TerminalProcess::running() const
{
    return m_process->state() != QProcess::NotRunning;
}

void TerminalProcess::start()
{
    if (running())
        return;

#ifdef Q_OS_WIN
    m_process->start("powershell.exe", {"-NoLogo", "-NoExit"});
#else
    m_process->start("/bin/sh", {"-i"});
#endif
}

void TerminalProcess::stop()
{
    if (!running())
        return;

    m_process->terminate();
    if (!m_process->waitForFinished(3000)) {
        m_process->kill();
        m_process->waitForFinished(2000);
    }
}

void TerminalProcess::write(const QString &text)
{
    if (!running())
        return;

    m_process->write((text + "\n").toUtf8());
}

void TerminalProcess::onReadyReadStdout()
{
    QString data = QString::fromUtf8(m_process->readAllStandardOutput());
    if (!data.isEmpty())
        emit outputReceived(data);
}

void TerminalProcess::onReadyReadStderr()
{
    QString data = QString::fromUtf8(m_process->readAllStandardError());
    if (!data.isEmpty())
        emit outputReceived(data);
}

void TerminalProcess::onProcessFinished(int exitCode, QProcess::ExitStatus status)
{
    Q_UNUSED(exitCode)
    Q_UNUSED(status)
    emit runningChanged();
}
