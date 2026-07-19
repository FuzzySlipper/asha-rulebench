import { INVALID_MISSING_SUPPORT_WORKSPACE } from '../../shared/rulebench-content.js';

// Evaluated by the authoring subprocess, but deliberately not exported from
// the selected root declaration. It therefore cannot enter package closure.
Object.isFrozen(INVALID_MISSING_SUPPORT_WORKSPACE.packages);
