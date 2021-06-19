//! Ceres Runtime
use crate::{storage::MemoryStorage, util, Error, Metadata, Result, Storage};
use ceres_executor::{Builder, Instance, Memory};
use ceres_sandbox::{Sandbox, Transaction};
use ceres_seal::RuntimeInterfaces;
use ceres_std::{Rc, String, ToString, Vec};
use core::cell::RefCell;
use parity_wasm::elements::Module;

/// Ceres Runtime
pub struct Runtime {
    pub sandbox: Rc<RefCell<Sandbox>>,
    instance: Instance<Sandbox>,
    pub metadata: Metadata,
    storage: Rc<RefCell<dyn Storage>>,
}

impl Runtime {
    /// Create runtime from contract
    pub fn from_contract(contract: &[u8], ri: Option<impl RuntimeInterfaces>) -> Result<Runtime> {
        let meta = serde_json::from_str::<Metadata>(&String::from_utf8_lossy(contract))
            .map_err(|_| Error::DecodeContractFailed)?;

        Self::new(
            &hex::decode(&meta.source.wasm.as_bytes()[2..])
                .map_err(|_| Error::DecodeContractFailed)?,
            meta,
            Rc::new(RefCell::new(MemoryStorage::new())),
            ri,
        )
    }

    /// Create runtime from contract
    pub fn from_contract_and_storage(
        contract: &[u8],
        storage: Rc<RefCell<impl Storage + 'static>>,
        ri: Option<impl RuntimeInterfaces>,
    ) -> Result<Runtime> {
        let meta = serde_json::from_str::<Metadata>(&String::from_utf8_lossy(contract))
            .map_err(|_| Error::DecodeContractFailed)?;

        Self::new(
            &hex::decode(&meta.source.wasm.as_bytes()[2..])
                .map_err(|_| Error::DecodeContractFailed)?,
            meta,
            storage,
            ri,
        )
    }

    /// Create runtime from metadata and storage
    pub fn from_metadata_and_storage(
        meta: Metadata,
        storage: Rc<RefCell<impl Storage + 'static>>,
        ri: Option<impl RuntimeInterfaces>,
    ) -> Result<Runtime> {
        Self::new(
            &hex::decode(&meta.source.wasm.as_bytes()[2..])
                .map_err(|_| Error::DecodeContractFailed)?,
            meta,
            storage,
            ri,
        )
    }

    /// New runtime
    pub fn new(
        b: &[u8],
        metadata: Metadata,
        storage: Rc<RefCell<impl Storage + 'static>>,
        ri: Option<impl RuntimeInterfaces>,
    ) -> Result<Runtime> {
        let mut el = Module::from_bytes(b).map_err(|_| Error::ParseWasmModuleFailed)?;
        if el.has_names_section() {
            el = match el.parse_names() {
                Ok(m) => m,
                Err((_, m)) => m,
            }
        }

        // Set memory
        let limit = util::scan_imports(&el).map_err(|_| Error::CalcuateMemoryLimitFailed)?;
        let mem = Memory::new(limit.0, limit.1).map_err(|_| Error::AllocMemoryFailed)?;

        // Get storage
        let storage_mut = storage.borrow_mut();
        let state =
            if let Some(state) = storage_mut.get(util::parse_code_hash(&metadata.source.hash)?) {
                state
            } else {
                storage_mut.new_state()
            };

        // Create Sandbox and Builder
        let sandbox = Rc::new(RefCell::new(Sandbox::new(mem, state)));

        // Construct interfaces
        let mut builder = Builder::new().add_host_parcels(ceres_seal::pallet_contracts(ri));

        // **Note**
        //
        // The memory is `cloned()`, trying using one memory.
        builder.add_memory("env", "memory", sandbox.borrow().mem());

        // Create instance
        let instance = Instance::new(
            &el.to_bytes()
                .map_err(|error| Error::SerializeFailed { error })?,
            &builder,
            &mut sandbox.borrow_mut(),
        )
        .map_err(|error| Error::InitModuleFailed { error })?;

        drop(storage_mut);
        Ok(Runtime {
            sandbox,
            instance,
            metadata,
            storage,
        })
    }

    /// Deploy contract
    pub fn deploy(&mut self, method: &str, args: &[&str], tx: Option<Transaction>) -> Result<()> {
        if let Some(tx) = tx {
            self.sandbox.borrow_mut().tx = tx;
        }

        let constructors = self.metadata.constructors();
        let (selector, tys) = constructors.get(method).ok_or(Error::GetMethodFailed {
            name: method.to_string(),
        })?;

        let mut bm = self.sandbox.borrow_mut();
        bm.input = Some(util::parse_args(
            selector,
            args,
            tys.iter().map(|ty| ty.1).collect(),
        )?);
        self.instance
            .invoke("deploy", &[], &mut bm)
            .map_err(|error| Error::DeployContractFailed { error })?;

        Ok(())
    }

    /// Call contract
    pub fn call(
        &mut self,
        method: &str,
        args: &[&str],
        tx: Option<Transaction>,
    ) -> Result<Vec<u8>> {
        if let Some(tx) = tx {
            self.sandbox.borrow_mut().tx = tx;
        }

        let messages = self.metadata.messages();
        let (selector, tys) = messages.get(method).ok_or(Error::GetMethodFailed {
            name: method.to_string(),
        })?;

        let mut bm = self.sandbox.borrow_mut();
        bm.input = Some(util::parse_args(
            selector,
            args,
            tys.iter().map(|ty| ty.1).collect(),
        )?);

        let res = self.instance.invoke("call", &[], &mut bm);
        if let Some(ret) = bm.ret.take() {
            return Ok(ret);
        } else {
            res.map_err(|error| Error::CallContractFailed { error })?;
        }

        Ok(vec![])
    }

    /// Flush storage
    pub fn flush(&mut self) -> Result<()> {
        self.storage.borrow_mut().set(
            util::parse_code_hash(&self.metadata.source.hash)?,
            self.sandbox.borrow().state.clone(),
        )?;

        Ok(())
    }
}
