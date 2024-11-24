#![no_std]
#![no_main]

use kernel::prelude::*;
use kernel::file_operations::{FileOperations, FileOpener};
use kernel::procfs::{ProcFile, ProcFileOperations};
use kernel::sync::Mutex;

use kernel::c_types::*;
use kernel::module_param::KernelParam;
use kernel::task::Task;

// Estrutura para armazenar o PID
struct ProcPid {
    pid: Mutex<Option<i32>>, // PID será armazenado em uma Mutex protegida
}

// Implementação para o módulo
impl KernelModule for ProcPid {
    fn init() -> Result<Self> {
        pr_info!("Modulo carregado: /proc/pid\n");
        // Criação do arquivo /proc/pid
        let proc_file = ProcFile::new_pinned("pid", FileOps)?;
        Ok(Self {
            pid: Mutex::new(None),
        })
    }
}

impl Drop for ProcPid {
    fn drop(&mut self) {
        pr_info!("Modulo removido: /proc/pid\n");
    }
}

// Operações de arquivo associadas
struct FileOps;

// Implementação das operações de leitura e escrita
impl FileOperations for FileOps {
    type Wrapper = ProcPid;

    // Função de escrita
    fn write(
        this: &Self::Wrapper,
        _file: &kernel::file_operations::File,
        data: &[u8],
        _offset: &mut u64,
    ) -> Result<usize> {
        let input = core::str::from_utf8(data).map_err(|_| Error::EINVAL)?;
        if let Ok(pid) = input.trim().parse::<i32>() {
            *this.pid.lock() = Some(pid); // Atualiza o PID na Mutex
            pr_info!("PID registrado: {}\n", pid);
        } else {
            pr_err!("Erro ao interpretar PID\n");
            return Err(Error::EINVAL);
        }
        Ok(data.len())
    }

    // Função de leitura
    fn read(
        this: &Self::Wrapper,
        _file: &kernel::file_operations::File,
        buffer: &mut [u8],
        _offset: &mut u64,
    ) -> Result<usize> {
        let pid = match *this.pid.lock() {
            Some(pid) => pid,
            None => {
                pr_info!("Nenhum PID definido\n");
                return Ok(0);
            }
        };

        // Localizar a tarefa associada ao PID
        let task = Task::from_pid(pid).ok_or(Error::EINVAL)?;
        let state = task.state();

        // Formata as informações para serem retornadas
        let info = format!(
            "command = [{}] pid = [{}] state = [{}]\n",
            task.comm(),
            pid,
            state as i32
        );

        // Copia os dados para o buffer de leitura
        let len = info.len().min(buffer.len());
        buffer[..len].copy_from_slice(&info.as_bytes()[..len]);
        Ok(len)
    }
}

// Registro do módulo
module! {
    type: ProcPid,
    name: b"proc_pid",
    author: b"Matheus",
    description: b"Modulo para exibir informações de uma tarefa usando /proc/pid",
    license: b"GPL",
}
