use crate::application::service::Service;
use crate::database::repository::Repository;
use crate::support::Logger;

pub struct Controller {
    service: Service,
    repository: Repository,
    logger: Logger,
}
