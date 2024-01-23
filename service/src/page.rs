use std::str::FromStr;

use dto::page::{Pageable, PageableOptions, SortDirection};
use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, FromQueryResult, PaginatorTrait, QueryOrder,
    QuerySelect, Select, SelectTwoMany,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub content: Vec<T>,
    pub last: bool,
    pub total_elements: u64,
    pub total_pages: u64,
    pub size: u64,
    pub number: u64,
    pub first: bool,
    pub number_of_elements: u64,
    pub empty: bool,
}

pub enum PageError {
    InvalidColumnName(String),
    InvalidDirectionName,
    DbErr(DbErr),
}

impl From<DbErr> for PageError {
    fn from(e: DbErr) -> Self {
        Self::DbErr(e)
    }
}

impl<T> Page<T> {
    pub async fn paginate<'db, E: EntityTrait<Model = T>>(
        db: &'db DatabaseConnection,
        operation: Select<E>,
        pageable: Option<Pageable>,
    ) -> Result<Self, PageError>
    where
        T: FromQueryResult + Sized + Send + Sync + 'db,
    {
        let Some(pageable) = pageable else {
            let items = operation.all(db).await?;
            let (number_of_elements, empty) = (items.len() as u64, items.is_empty());
            return Ok(Self {
                content: items,
                last: true,
                total_elements: number_of_elements,
                total_pages: 1,
                size: number_of_elements,
                number: 0,
                first: true,
                number_of_elements,
                empty,
            });
        };

        let mut operation = operation;

        if let Some(sort) = pageable.sort {
            for criterion in sort.criteria {
                operation = operation.order_by(
                    E::Column::from_str(&criterion.field)
                        .map_err(|_| PageError::InvalidColumnName(criterion.field))?,
                    match criterion.direction {
                        SortDirection::Asc => sea_orm::Order::Asc,
                        SortDirection::Desc => sea_orm::Order::Desc,
                    },
                );
            }
        }

        let operation = operation;

        match pageable.options {
            PageableOptions::OffsetAndLimit((offset, limit)) => {
                let items = operation
                    .offset(offset)
                    .limit(limit)
                    .paginate(db, limit)
                    .fetch()
                    .await?;

                Ok(Self {
                    content: items,
                    last: true,
                    total_elements: 0,
                    total_pages: 0,
                    size: 0,
                    number: 0,
                    first: false,
                    number_of_elements: 0,
                    empty: false,
                })
            }
            PageableOptions::Page(page) => {
                let paginator = operation.paginate(db, page.per_page);
                let items = paginator.fetch().await?;

                let num_items_and_pages = paginator.num_items_and_pages().await?;

                let (number_of_elements, empty) = (items.len() as u64, items.is_empty());

                Ok(Self {
                    content: items,
                    last: page.index == num_items_and_pages.number_of_pages - 1,
                    total_elements: num_items_and_pages.number_of_items,
                    total_pages: num_items_and_pages.number_of_pages,
                    size: page.per_page,
                    number: page.index,
                    first: page.index == 0,
                    number_of_elements,
                    empty,
                })
            }
        }
    }

    pub fn map<U>(self, f: impl Fn(T) -> U) -> Page<U> {
        Page {
            content: self.content.into_iter().map(f).collect(),
            last: self.last,
            total_elements: self.total_elements,
            total_pages: self.total_pages,
            size: self.size,
            number: self.number,
            first: self.first,
            number_of_elements: self.number_of_elements,
            empty: self.empty,
        }
    }
}

impl<T, U> Page<(T, Vec<U>)> {
    pub async fn paginate_two_many<'db, E: EntityTrait<Model = T>, F: EntityTrait<Model = U>>(
        db: &'db DatabaseConnection,
        operation: SelectTwoMany<E, F>,
        pageable: Option<Pageable>,
    ) -> Result<Self, PageError>
    where
        T: FromQueryResult + Sized + Send + Sync + 'db,
        U: FromQueryResult + Sized + Send + Sync + 'db,
    {
        let Some(pageable) = pageable else {
            let items = operation.all(db).await?;
            let (number_of_elements, empty) = (items.len() as u64, items.is_empty());
            return Ok(Self {
                content: items,
                last: true,
                total_elements: number_of_elements,
                total_pages: 1,
                size: number_of_elements,
                number: 0,
                first: true,
                number_of_elements,
                empty,
            });
        };

        let mut operation = operation;

        if let Some(sort) = pageable.sort {
            for criterion in sort.criteria {
                operation = operation.order_by(
                    E::Column::from_str(&criterion.field)
                        .map_err(|_| PageError::InvalidColumnName(criterion.field))?,
                    match criterion.direction {
                        SortDirection::Asc => sea_orm::Order::Asc,
                        SortDirection::Desc => sea_orm::Order::Desc,
                    },
                );
            }
        }

        let operation = operation;

        match pageable.options {
            PageableOptions::OffsetAndLimit((offset, limit)) => {
                let items = operation.offset(offset).limit(limit).all(db).await?;

                let total_elements = items.len() as u64;

                Ok(Self {
                    content: items,
                    last: true,
                    total_elements,
                    total_pages: 1,
                    size: 0,
                    number: 0,
                    first: true,
                    number_of_elements: total_elements,
                    empty: total_elements == 0,
                })
            }
            PageableOptions::Page(page) => {
                let items = operation
                    .offset(page.index * page.per_page)
                    .limit(page.per_page)
                    .all(db)
                    .await?;

                let total_elements = items.len() as u64;

                Ok(Self {
                    content: items,
                    last: false,
                    total_elements,
                    total_pages: 1,
                    size: page.per_page,
                    number: page.index,
                    first: page.index == 0,
                    number_of_elements: total_elements,
                    empty: total_elements == 0,
                })
            }
        }
    }
}

impl<T, U> From<Page<T>> for dto::page::Page<U>
where
    U: From<T>,
{
    fn from(page: Page<T>) -> Self {
        Self {
            content: page.content.into_iter().map(U::from).collect(),
            last: page.last,
            total_elements: page.total_elements,
            total_pages: page.total_pages,
            size: page.size,
            number: page.number,
            first: page.first,
            number_of_elements: page.number_of_elements,
            empty: page.empty,
        }
    }
}
